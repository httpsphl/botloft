//! Reading a repository with `git` (spec 33.5): where its top folder is,
//! the branch, and the changed files with their lines added and removed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use botloft_core::protocol::{GitFile, GitFileStatus, GitRepo};

use crate::platform;

/// The longest one `git` call may take.
const LIMIT: Duration = Duration::from_secs(5);
/// The most files listed for one repository.
pub(super) const MAX_FILES: usize = 200;

/// Status without taking git's lock, so the agent's own git calls never
/// wait for it, and never asking for a password.
const ENV: [(&str, &str); 2] = [("GIT_OPTIONAL_LOCKS", "0"), ("GIT_TERMINAL_PROMPT", "0")];

fn git(cwd: &Path, args: &[&str]) -> Option<String> {
    let output = platform::run_helper("git", args, cwd, &ENV, LIMIT).ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The top folder of the repository `folder` is in; `None` when it is in
/// none, or git is not installed.
pub(super) fn top(folder: &Path) -> Option<PathBuf> {
    if !folder.is_dir() {
        return None;
    }
    let out = git(folder, &["rev-parse", "--show-toplevel"])?;
    let top = out.trim();
    (!top.is_empty()).then(|| PathBuf::from(top))
}

/// The repository's branch and changed files.
pub(super) fn read(top: &Path) -> Option<GitRepo> {
    let status = git(
        top,
        &[
            "-c",
            "core.quotepath=false",
            "status",
            "--porcelain=v2",
            "--branch",
            "--untracked-files=normal",
        ],
    )?;
    // A repository with no commit has no HEAD to compare with.
    let lines = git(
        top,
        &["-c", "core.quotepath=false", "diff", "--numstat", "HEAD"],
    )
    .map(|numstat| counts(&numstat))
    .unwrap_or_default();
    let mut repo = parse_status(&status, &lines);
    repo.path = top.to_string_lossy().into_owned();
    repo.name = top.file_name().map_or_else(
        || repo.path.clone(),
        |name| name.to_string_lossy().into_owned(),
    );
    Some(repo)
}

/// Lines added and removed by path, from `git diff --numstat`; `None` for
/// a binary file.
pub(super) fn counts(numstat: &str) -> HashMap<String, Option<(u32, u32)>> {
    numstat
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let added = fields.next()?;
            let removed = fields.next()?;
            let path = unquote(fields.next()?);
            let pair = added.parse().ok().zip(removed.parse().ok());
            Some((path, pair))
        })
        .collect()
}

/// The branch and the files of `git status --porcelain=v2 --branch`.
pub(super) fn parse_status(status: &str, lines: &HashMap<String, Option<(u32, u32)>>) -> GitRepo {
    let mut repo = GitRepo {
        path: String::new(),
        name: String::new(),
        branch: None,
        ahead: 0,
        behind: 0,
        files: Vec::new(),
        more: 0,
    };
    for line in status.lines() {
        if let Some(head) = line.strip_prefix("# branch.head ") {
            repo.branch = (head != "(detached)").then(|| head.to_owned());
        } else if let Some(ab) = line.strip_prefix("# branch.ab ") {
            let mut parts = ab.split(' ');
            repo.ahead = number(parts.next(), '+');
            repo.behind = number(parts.next(), '-');
        } else if let Some((path, status)) = changed(line) {
            if repo.files.len() >= MAX_FILES {
                repo.more += 1;
                continue;
            }
            let pair = if status == GitFileStatus::Untracked {
                None
            } else {
                lines.get(&path).copied().flatten()
            };
            repo.files.push(GitFile {
                path,
                status,
                added: pair.map(|(added, _)| added),
                removed: pair.map(|(_, removed)| removed),
            });
        }
    }
    repo
}

fn number(part: Option<&str>, sign: char) -> u32 {
    part.and_then(|part| part.strip_prefix(sign))
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

/// A changed entry of the porcelain: its path and how it changed.
fn changed(line: &str) -> Option<(String, GitFileStatus)> {
    let kind = line.split(' ').next()?;
    let (fields, xy) = match kind {
        "1" => (9, line.split(' ').nth(1)?),
        "2" => (10, line.split(' ').nth(1)?),
        "u" => {
            return Some((
                unquote(line.splitn(11, ' ').nth(10)?),
                GitFileStatus::Conflicted,
            ));
        }
        "?" => return Some((unquote(line.get(2..)?), GitFileStatus::Untracked)),
        _ => return None,
    };
    let rest = line.splitn(fields, ' ').nth(fields - 1)?;
    // A rename gives the new path, a tab, then the old one.
    let path = rest.split('\t').next()?;
    let status = if xy.contains('R') || xy.contains('C') {
        GitFileStatus::Renamed
    } else if xy.contains('A') {
        GitFileStatus::Added
    } else if xy.contains('D') {
        GitFileStatus::Deleted
    } else {
        GitFileStatus::Modified
    };
    Some((unquote(path), status))
}

/// Git quotes a path with unusual characters; the quotes go, the rest
/// stays as written.
fn unquote(path: &str) -> String {
    path.strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(path)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_porcelain_gives_the_branch_and_each_change() {
        let status = "\
# branch.oid 1234567
# branch.head feat/site
# branch.upstream origin/feat/site
# branch.ab +2 -1
1 .M N... 100644 100644 100644 aaa bbb src/app.ts
1 A. N... 000000 100644 100644 000 ccc docs/new guide.md
1 .D N... 100644 100644 000000 ddd ddd old.txt
2 R. N... 100644 100644 100644 eee eee R100 src/main.ts\tsrc/index.ts
u UU N... 100644 100644 100644 100644 f1 f2 f3 conflict.rs
? notes/todo.md
";
        let lines = counts("12\t3\tsrc/app.ts\n40\t0\tdocs/new guide.md\n-\t-\timage.png\n");
        let repo = parse_status(status, &lines);
        assert_eq!(repo.branch.as_deref(), Some("feat/site"));
        assert_eq!((repo.ahead, repo.behind), (2, 1));
        let seen: Vec<_> = repo
            .files
            .iter()
            .map(|file| (file.path.as_str(), file.status, file.added, file.removed))
            .collect();
        assert_eq!(
            seen,
            [
                ("src/app.ts", GitFileStatus::Modified, Some(12), Some(3)),
                ("docs/new guide.md", GitFileStatus::Added, Some(40), Some(0)),
                ("old.txt", GitFileStatus::Deleted, None, None),
                ("src/main.ts", GitFileStatus::Renamed, None, None),
                ("conflict.rs", GitFileStatus::Conflicted, None, None),
                ("notes/todo.md", GitFileStatus::Untracked, None, None),
            ]
        );
        assert_eq!(
            lines.get("image.png"),
            Some(&None),
            "a binary file has no counts"
        );
    }

    #[test]
    fn a_detached_head_has_no_branch_and_long_lists_are_cut() {
        let mut status = String::from("# branch.head (detached)\n");
        for n in 0..MAX_FILES + 3 {
            status.push_str(&format!("? file{n}.txt\n"));
        }
        let repo = parse_status(&status, &HashMap::new());
        assert_eq!(repo.branch, None);
        assert_eq!(repo.files.len(), MAX_FILES);
        assert_eq!(repo.more, 3);
    }
}
