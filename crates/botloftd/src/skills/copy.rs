//! Copies of skills (spec 38.3): Botloft's own copy of each skill given to
//! an agent, and the copy in the agent's `.claude\skills`. A copy never
//! follows a link inside the skill, and has a mark so Botloft only ever
//! replaces or removes what it put there.

use std::io;
use std::path::Path;

use tracing::warn;

/// The file that marks a copy as Botloft's.
pub(super) const MARK: &str = ".botloft-skill";
/// Most files and bytes one skill may hold.
const MAX_FILES: usize = 2000;
const MAX_BYTES: u64 = 50 * 1024 * 1024;

/// Puts a fresh copy of the skill in `from` at `to`, marked. Whatever was
/// at `to` goes: callers check that it was Botloft's.
pub(super) fn copy_skill(from: &Path, to: &Path) -> io::Result<()> {
    // Next to it, with a dot, so nothing loads it as a skill meanwhile.
    let name = to.file_name().unwrap_or_default().to_string_lossy();
    let staging = to.with_file_name(format!(".{name}.botloft-new"));
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    let mut budget = (MAX_FILES, MAX_BYTES);
    let copied = copy_dir(from, &staging, &mut budget)
        .and_then(|()| std::fs::write(staging.join(MARK), b"Copied by Botloft (spec 38.3).\n"));
    if let Err(err) = copied {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(err);
    }
    if to.exists() {
        std::fs::remove_dir_all(to)?;
    }
    std::fs::rename(&staging, to)
}

fn copy_dir(from: &Path, to: &Path, budget: &mut (usize, u64)) -> io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        // A link inside a skill could point anywhere: it is left out.
        if kind.is_symlink() {
            continue;
        }
        let name = entry.file_name();
        if name == MARK || name == ".git" {
            continue;
        }
        let target = to.join(&name);
        if kind.is_dir() {
            copy_dir(&entry.path(), &target, budget)?;
        } else if kind.is_file() {
            let size = entry.metadata()?.len();
            if budget.0 == 0 || size > budget.1 {
                return Err(io::Error::other("the skill is too big to copy"));
            }
            budget.0 -= 1;
            budget.1 -= size;
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

pub(super) fn is_ours(dir: &Path) -> bool {
    dir.join(MARK).is_file()
}

/// The agent's `.claude\skills` holds Botloft's copies of `chosen`, from
/// `library`, and none of the others Botloft put there. A folder of the
/// same name that the agent made stays as it is.
pub(super) fn sync(library: &Path, workspace: &Path, chosen: &[String]) -> io::Result<()> {
    let dir = workspace.join(".claude").join("skills");
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            if is_ours(&path) && !chosen.contains(&name) {
                std::fs::remove_dir_all(&path)?;
            }
        }
    }
    for name in chosen {
        let from = library.join(name);
        if !from.is_dir() {
            continue;
        }
        let to = dir.join(name);
        if to.exists() && !is_ours(&to) {
            warn!("an agent has a skill folder of its own with the name of one it was given");
            continue;
        }
        std::fs::create_dir_all(&dir)?;
        copy_skill(&from, &to)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(dir: &Path, text: &str) {
        std::fs::create_dir_all(dir.join("scripts")).expect("dir");
        std::fs::write(dir.join("SKILL.md"), text).expect("skill");
        std::fs::write(dir.join("scripts").join("run.py"), "print(1)").expect("script");
    }

    #[test]
    fn the_agent_gets_marked_copies_and_keeps_its_own_folders() {
        let root = tempfile::tempdir().expect("dir");
        let library = root.path().join("library");
        let workspace = root.path().join("bot");
        skill(&library.join("deploy"), "deploy v1");
        skill(&library.join("review"), "review");
        // A skill the agent made itself, with a name it was given too.
        skill(&workspace.join(".claude/skills/review"), "its own");
        sync(&library, &workspace, &["deploy".into(), "review".into()]).expect("sync");
        let deploy = workspace.join(".claude/skills/deploy");
        assert!(is_ours(&deploy));
        assert!(deploy.join("scripts/run.py").is_file());
        let own = workspace.join(".claude/skills/review/SKILL.md");
        assert_eq!(std::fs::read_to_string(own).expect("own"), "its own");

        // A new version reaches the agent; a skill taken back goes.
        std::fs::write(library.join("deploy/SKILL.md"), "deploy v2").expect("v2");
        sync(&library, &workspace, &["deploy".into()]).expect("again");
        assert_eq!(
            std::fs::read_to_string(deploy.join("SKILL.md")).expect("v2"),
            "deploy v2"
        );
        sync(&library, &workspace, &[]).expect("none");
        assert!(!deploy.exists());
        assert!(workspace.join(".claude/skills/review").is_dir());
    }
}
