//! Walking a folder for the files a bot made (spec 8.5): bounded, so a work
//! folder as big as `Documents` costs little, and never through links.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Folders below the top that are never files a bot made for the owner.
const SKIP_DIRS: [&str; 5] = [
    "node_modules",
    "target",
    "__pycache__",
    "venv",
    "site-packages",
];
const MAX_DEPTH: usize = 6;
const MAX_VISITED: usize = 20_000;

/// A file found: its path, size and modification time (Unix ms).
pub(super) struct Found {
    pub path: PathBuf,
    pub size: u64,
    pub modified_at: i64,
}

/// What to leave out at the top of a root: the workspace holds the rules and
/// memory the daemon writes and the files the owner sent.
pub(super) struct Root<'a> {
    pub path: &'a Path,
    pub skip_top: &'a [&'a str],
}

/// Files below `root` modified at or after `since` (Unix ms), in no order.
/// Hidden entries, links and generated folders are left out.
pub(super) fn walk(root: &Root<'_>, since: i64) -> Vec<Found> {
    let mut found = Vec::new();
    let mut seen = 0;
    let mut pending = vec![(root.path.to_path_buf(), 0)];
    while let Some((dir, depth)) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            seen += 1;
            if seen > MAX_VISITED {
                return found;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let left_out = depth == 0 && root.skip_top.contains(&name.as_str());
            if name.starts_with('.') || name.starts_with("~$") || left_out {
                continue;
            }
            // `DirEntry::metadata` does not follow links.
            let Ok(meta) = entry.metadata() else { continue };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                if depth < MAX_DEPTH && !SKIP_DIRS.contains(&name.as_str()) {
                    pending.push((entry.path(), depth + 1));
                }
            } else if meta.is_file() {
                let modified_at = millis(&meta);
                if modified_at >= since {
                    found.push(Found {
                        path: entry.path(),
                        size: meta.len(),
                        modified_at,
                    });
                }
            }
        }
    }
    found
}

/// Modification time of `meta` in Unix milliseconds.
pub(super) fn millis(meta: &std::fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |since| {
            i64::try_from(since.as_millis()).unwrap_or(i64::MAX)
        })
}

/// One key for a path however it is spelled: Windows paths do not care about
/// case or the direction of slashes.
pub(super) fn key(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text
    }
}

/// The paths of `files` as keys, to ask "did the bot write this?".
pub(super) fn keys(files: &[String]) -> HashSet<String> {
    files.iter().map(|file| key(Path::new(file))).collect()
}

/// The media type for a file name, by extension.
pub(crate) fn media_type(name: &str) -> &'static str {
    let extension = name
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "txt" | "log" => "text/plain",
        "md" | "markdown" => "text/markdown",
        "csv" => "text/csv",
        "html" | "htm" => "text/html",
        "json" => "application/json",
        "xml" => "application/xml",
        "yml" | "yaml" => "application/yaml",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "zip" => "application/zip",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        // Source and config files read as text.
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "css" | "toml" | "ini" | "sql" | "sh"
        | "ps1" | "bat" | "java" | "go" | "c" | "h" | "cpp" | "cs" | "rb" | "php" => "text/plain",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extensions_give_media_types() {
        assert_eq!(media_type("Report.PDF"), "application/pdf");
        assert_eq!(media_type("notes.md"), "text/markdown");
        assert_eq!(media_type("script.ps1"), "text/plain");
        assert_eq!(media_type("noextension"), "application/octet-stream");
    }

    #[test]
    fn a_walk_skips_hidden_generated_and_top_level_leftovers() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        for file in [
            "a.md",
            "sub/deep/b.pdf",
            ".hidden.txt",
            ".git/config",
            "node_modules/x.js",
            "attachments/2026-01-01/mine.png",
            "CLAUDE.md",
            "~$lock.docx",
        ] {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
            std::fs::write(path, "x").expect("write");
        }
        let skip = ["attachments", "CLAUDE.md"];
        let top = Root {
            path: root,
            skip_top: &skip,
        };
        let mut names: Vec<_> = walk(&top, 0)
            .into_iter()
            .map(|file| key(file.path.strip_prefix(root).expect("inside")))
            .collect();
        names.sort();
        assert_eq!(names, ["a.md", "sub/deep/b.pdf"]);
        let all = Root {
            path: root,
            skip_top: &[],
        };
        assert!(walk(&all, i64::MAX).is_empty(), "nothing is that new");
    }
}
