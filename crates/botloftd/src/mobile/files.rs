//! Files a phone sends (spec 28.14). The phone seals each one with a key of
//! its own and leaves it on the server; the key comes inside the sealed
//! `send`. The computer takes the file, opens it, hands it on as an
//! attachment the way the app does, and the server forgets it.

use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD as B64};
use botloft_core::protocol::{AttachmentUpload, PHONE_FILE_MAX, PhoneFile};
use botloft_core::validate::ATTACHMENTS_MAX;

use super::identity;
use super::seal::SealError;
use crate::state::Daemon;

/// What every file's seal is bound to. One key seals one file only, so the
/// nonce can be the same for all.
const AAD: &[u8] = b"botloft-file-1";
const NONCE: [u8; 12] = [0; 12];
/// The seal's tag, at the end of what the server keeps.
const TAG: u64 = 16;

fn cipher(key: &[u8; 32]) -> Result<Aes256Gcm, SealError> {
    Aes256Gcm::new_from_slice(key).map_err(|_| SealError::Key)
}

/// A file as the phone seals it (for the test vectors).
pub fn seal_file(key: &[u8; 32], plain: &[u8]) -> Result<Vec<u8>, SealError> {
    cipher(key)?
        .encrypt(
            &NONCE.into(),
            Payload {
                msg: plain,
                aad: AAD,
            },
        )
        .map_err(|_| SealError::Auth)
}

/// The file inside, if it opens with `key`.
pub fn open_file(key: &[u8; 32], sealed: &[u8]) -> Result<Vec<u8>, SealError> {
    cipher(key)?
        .decrypt(
            &NONCE.into(),
            Payload {
                msg: sealed,
                aad: AAD,
            },
        )
        .map_err(|_| SealError::Auth)
}

/// The server's name for a file: `fil_` and plain letters, so it is a path
/// part and nothing else.
fn plain_id(id: &str) -> bool {
    id.len() <= 40
        && id.strip_prefix("fil_").is_some_and(|rest| {
            !rest.is_empty()
                && rest
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
}

fn key_of(file: &PhoneFile) -> Option<[u8; 32]> {
    B64.decode(&file.key).ok()?.try_into().ok()
}

/// The files of a `send`, opened, as the app would attach them; or the
/// reason, in the words the phone knows. Taken or not, the server forgets
/// them: a phone that tries again sends them again.
pub(crate) async fn take(
    daemon: &Daemon,
    files: &[PhoneFile],
) -> Result<Vec<AttachmentUpload>, &'static str> {
    if files.is_empty() {
        return Ok(Vec::new());
    }
    let most = PHONE_FILE_MAX.min(daemon.bots.attachment_max_bytes);
    let fits = files.len() <= ATTACHMENTS_MAX
        && files.iter().all(|file| {
            plain_id(&file.id) && key_of(file).is_some() && file.size > 0 && file.size <= most
        });
    if !fits {
        return Err("invalid");
    }
    let (server, credentials) = identity(daemon).map_err(|_| "failed")?;
    let mut uploads = Vec::with_capacity(files.len());
    let mut failed = None;
    for file in files {
        let Some(key) = key_of(file) else {
            return Err("invalid");
        };
        let opened = match server
            .phone_file(&credentials.token, &file.id, file.size + TAG)
            .await
        {
            Ok(sealed) => open_file(&key, &sealed).ok(),
            Err(_) => None,
        };
        match opened {
            Some(bytes) if bytes.len() as u64 == file.size => uploads.push(AttachmentUpload {
                name: file.name.clone(),
                media_type: file.media_type.clone(),
                data: STANDARD.encode(bytes),
            }),
            _ => {
                failed = Some("failed");
                break;
            }
        }
    }
    for file in files.iter().filter(|file| plain_id(&file.id)) {
        let _ = server.delete_phone_file(&credentials.token, &file.id).await;
    }
    match failed {
        Some(reason) => Err(reason),
        None => Ok(uploads),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_opens_only_with_its_own_key_and_whole() {
        let key = [9u8; 32];
        let sealed = seal_file(&key, b"a receipt").expect("seal");
        assert_eq!(sealed.len() as u64, 9 + TAG);
        assert_eq!(open_file(&key, &sealed).expect("open"), b"a receipt");
        assert_eq!(open_file(&[8u8; 32], &sealed).err(), Some(SealError::Auth));
        let mut touched = sealed.clone();
        touched[0] ^= 1;
        assert_eq!(open_file(&key, &touched).err(), Some(SealError::Auth));
        assert_eq!(
            open_file(&key, &sealed[..sealed.len() - 1]).err(),
            Some(SealError::Auth)
        );
    }

    #[test]
    fn only_a_server_name_is_taken_as_a_file_id() {
        assert!(plain_id("fil_AbC-9_x"));
        for bad in [
            "",
            "fil_",
            "cpy_abc",
            "fil_../x",
            "fil_a/b",
            "fil_a?b=1",
            &"fil_".repeat(11),
        ] {
            assert!(!plain_id(bad), "{bad}");
        }
    }
}
