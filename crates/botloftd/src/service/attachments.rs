//! Files the owner sends with a message (spec 9.5): saved in the bot's
//! `attachments\<yyyy-mm-dd>\` folder before the message is stored.

use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::ids::AttachmentId;
use botloft_core::protocol::{Attachment, AttachmentUpload};
use botloft_core::{slug, validate};

use super::{ApiError, ApiResult};

const ATTACHMENTS_DIR: &str = "attachments";
const DEFAULT_MEDIA_TYPE: &str = "application/octet-stream";

/// Checks and saves every upload. Returns what the message records.
pub(crate) fn save(
    workspace: &Path,
    uploads: Vec<AttachmentUpload>,
    max_bytes: u64,
    now_ms: i64,
) -> ApiResult<Vec<Attachment>> {
    if uploads.len() > validate::ATTACHMENTS_MAX {
        return Err(ApiError::validation(format!(
            "a message can carry at most {} files",
            validate::ATTACHMENTS_MAX
        )));
    }
    // Decode and check everything before writing anything.
    let mut files = Vec::with_capacity(uploads.len());
    for upload in uploads {
        let name = slug::file_name(&upload.name);
        let bytes = BASE64
            .decode(upload.data.as_bytes())
            .map_err(|_| ApiError::validation(format!("{name} is not valid base64")))?;
        let size = bytes.len() as u64;
        if size > max_bytes {
            return Err(ApiError::validation(format!(
                "{name} is larger than {} MB",
                max_bytes / (1024 * 1024)
            )));
        }
        files.push((name, media_type(&upload.media_type), bytes));
    }
    if files.is_empty() {
        return Ok(Vec::new());
    }
    let folder = format!("{ATTACHMENTS_DIR}/{}", date(now_ms));
    let dir = workspace.join(&folder);
    std::fs::create_dir_all(&dir).map_err(ApiError::Workspace)?;
    let mut saved = Vec::with_capacity(files.len());
    for (name, media_type, bytes) in files {
        let name = free_name(&dir, &name);
        std::fs::write(dir.join(&name), &bytes).map_err(ApiError::Workspace)?;
        saved.push(Attachment {
            id: AttachmentId::generate(),
            path: format!("{folder}/{name}"),
            name,
            media_type,
            size: bytes.len() as u64,
        });
    }
    Ok(saved)
}

/// A MIME type as given, or the generic one when it looks wrong.
fn media_type(given: &str) -> String {
    let given = given.trim().to_ascii_lowercase();
    let valid = given.len() <= 100
        && given.split_once('/').is_some_and(|(kind, sub)| {
            let token = |s: &str| {
                !s.is_empty()
                    && s.chars()
                        .all(|ch| ch.is_ascii_alphanumeric() || "+-.".contains(ch))
            };
            token(kind) && token(sub)
        });
    if valid {
        given
    } else {
        DEFAULT_MEDIA_TYPE.to_owned()
    }
}

/// `name`, or `name-2.ext`, `name-3.ext`... when the folder has it already.
fn free_name(dir: &Path, name: &str) -> String {
    if !dir.join(name).exists() {
        return name.to_owned();
    }
    let (stem, extension) = match name.rfind('.') {
        Some(dot) if dot > 0 => (&name[..dot], &name[dot..]),
        _ => (name, ""),
    };
    (2u32..)
        .map(|n| format!("{stem}-{n}{extension}"))
        .find(|candidate| !dir.join(candidate).exists())
        .unwrap_or_else(|| name.to_owned())
}

/// `yyyy-mm-dd` of a Unix time in milliseconds, in UTC.
fn date(ms: i64) -> String {
    let days = ms.div_euclid(86_400_000);
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upload(name: &str, media_type: &str, data: &[u8]) -> AttachmentUpload {
        AttachmentUpload {
            name: name.into(),
            media_type: media_type.into(),
            data: BASE64.encode(data),
        }
    }

    #[test]
    fn dates_are_utc_calendar_days() {
        assert_eq!(date(0), "1970-01-01");
        assert_eq!(date(1_790_000_000_000), "2026-09-21");
        assert_eq!(date(951_782_400_000), "2000-02-29");
    }

    #[test]
    fn uploads_are_saved_under_the_day_with_unique_names() {
        let dir = tempfile::tempdir().expect("tempdir");
        let now = 1_790_000_000_000;
        let first = save(
            dir.path(),
            vec![upload("plan.pdf", "application/pdf", b"one")],
            1024,
            now,
        )
        .expect("save");
        let second = save(
            dir.path(),
            vec![upload("../plan.pdf", "weird type", b"two")],
            1024,
            now,
        )
        .expect("save again");
        assert_eq!(first[0].path, "attachments/2026-09-21/plan.pdf");
        assert_eq!(second[0].path, "attachments/2026-09-21/plan-2.pdf");
        assert_eq!(second[0].media_type, DEFAULT_MEDIA_TYPE);
        assert_eq!(second[0].size, 3);
        let written = std::fs::read(dir.path().join(&second[0].path)).expect("read");
        assert_eq!(written, b"two");
    }

    #[test]
    fn too_many_too_big_or_broken_uploads_are_refused_whole() {
        let dir = tempfile::tempdir().expect("tempdir");
        let many = (0..=validate::ATTACHMENTS_MAX)
            .map(|n| upload(&format!("{n}.txt"), "text/plain", b"x"))
            .collect();
        assert!(save(dir.path(), many, 1024, 0).is_err());
        let big = vec![
            upload("ok.txt", "text/plain", b"x"),
            upload("big.bin", "", &[0; 11]),
        ];
        assert!(save(dir.path(), big, 10, 0).is_err());
        assert!(!dir.path().join("attachments").exists(), "nothing written");
        let broken = vec![AttachmentUpload {
            name: "a.png".into(),
            media_type: "image/png".into(),
            data: "not base64!".into(),
        }];
        assert!(save(dir.path(), broken, 1024, 0).is_err());
    }
}
