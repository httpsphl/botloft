//! Copies of plugins (spec 41.3): Botloft's own copy of each plugin given to
//! an agent, and the copy in the agent's `.claude\plugins`. A copy never
//! follows a link, leaves the plugin's MCP servers out, and has a mark with
//! a print of what it was copied from, so an unchanged plugin is not copied
//! again and Botloft only ever replaces or removes what it put there.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde_json::Value;
use tracing::warn;

/// The file that marks a copy as Botloft's.
const MARK: &str = ".botloft-plugin";
/// Most files and bytes one plugin may hold.
const MAX_FILES: usize = 5000;
const MAX_BYTES: u64 = 100 * 1024 * 1024;
/// Left out wherever they are: links are left out too.
const NEVER: [&str; 2] = [MARK, ".git"];
/// Left out at the plugin's root: Claude Code's own marks, and the MCP
/// servers, which agents do not get (spec 41.4).
const NOT_AT_ROOT: [&str; 3] = [".in_use", ".orphaned_at", ".mcp.json"];

fn left_out(name: &str, at_root: bool) -> bool {
    NEVER.contains(&name) || (at_root && NOT_AT_ROOT.contains(&name))
}

/// A print of the plugin in `dir`: every file's path, size and time. Two
/// copies of the same plugin have the same print.
fn print(dir: &Path) -> io::Result<String> {
    let mut hasher = DefaultHasher::new();
    walk(dir, dir, &mut hasher)?;
    Ok(format!("{:016x}", hasher.finish()))
}

fn walk(root: &Path, dir: &Path, hasher: &mut DefaultHasher) -> io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let kind = entry.file_type()?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if kind.is_symlink() || left_out(&name, dir == root) {
            continue;
        }
        let path = entry.path();
        path.strip_prefix(root).unwrap_or(&path).hash(hasher);
        if kind.is_dir() {
            walk(root, &path, hasher)?;
        } else {
            let meta = entry.metadata()?;
            meta.len().hash(hasher);
            let time = meta
                .modified()
                .ok()
                .and_then(|at| at.duration_since(UNIX_EPOCH).ok());
            time.map(|at| at.as_millis()).hash(hasher);
        }
    }
    Ok(())
}

fn mark_of(dir: &Path) -> Option<String> {
    std::fs::read_to_string(dir.join(MARK)).ok()
}

pub(super) fn is_ours(dir: &Path) -> bool {
    dir.join(MARK).is_file()
}

/// Puts a copy of the plugin in `from` at `to`, marked, unless `to` already
/// is a copy of it as it is now. Whatever was at `to` goes: callers check
/// that it was Botloft's.
pub(super) fn copy_plugin(from: &Path, to: &Path) -> io::Result<()> {
    let print = print(from)?;
    let mark = format!("Copied by Botloft (spec 41.3).\nfrom {print}\n");
    if mark_of(to).as_deref() == Some(mark.as_str()) {
        return Ok(());
    }
    // Next to it, with a dot, so a half-made copy is never taken for it.
    let name = to.file_name().unwrap_or_default().to_string_lossy();
    let staging = to.with_file_name(format!(".{name}.botloft-new"));
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    let mut budget = (MAX_FILES, MAX_BYTES);
    let copied = copy_dir(from, from, &staging, &mut budget)
        .and_then(|()| without_servers(&staging))
        .and_then(|()| std::fs::write(staging.join(MARK), &mark));
    if let Err(err) = copied {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(err);
    }
    if to.exists() {
        std::fs::remove_dir_all(to)?;
    }
    std::fs::rename(&staging, to)
}

fn copy_dir(root: &Path, from: &Path, to: &Path, budget: &mut (usize, u64)) -> io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let name = entry.file_name();
        // A link inside a plugin could point anywhere: it is left out.
        if kind.is_symlink() || left_out(&name.to_string_lossy(), from == root) {
            continue;
        }
        let target = to.join(&name);
        if kind.is_dir() {
            copy_dir(root, &entry.path(), &target, budget)?;
        } else if kind.is_file() {
            let size = entry.metadata()?.len();
            if budget.0 == 0 || size > budget.1 {
                return Err(io::Error::other("the plugin is too big to copy"));
            }
            budget.0 -= 1;
            budget.1 -= size;
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// The copy's manifest without the MCP servers it declares, nor the
/// channels bound to them.
fn without_servers(dir: &Path) -> io::Result<()> {
    let path = dir.join(".claude-plugin").join("plugin.json");
    let Ok(bytes) = std::fs::read(&path) else {
        return Ok(());
    };
    let Ok(Value::Object(mut manifest)) = serde_json::from_slice::<Value>(&bytes) else {
        // Claude Code will say what is wrong with it (spec 41.4).
        return Ok(());
    };
    let servers = manifest.remove("mcpServers");
    let channels = manifest.remove("channels");
    if servers.is_some() || channels.is_some() {
        let text = serde_json::to_vec_pretty(&manifest).map_err(io::Error::other)?;
        std::fs::write(&path, text)?;
    }
    Ok(())
}

/// The agent's `.claude\plugins` holds Botloft's copies of `chosen`, from
/// `library`, and none of the others Botloft put there. A folder of the
/// same name that the agent made stays as it is. Answers the folders the
/// agent loads with `--plugin-dir`.
pub(super) fn sync(
    library: &Path,
    workspace: &Path,
    chosen: &[String],
) -> io::Result<Vec<PathBuf>> {
    let dir = workspace.join(".claude").join("plugins");
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            if is_ours(&path) && !chosen.contains(&name) {
                std::fs::remove_dir_all(&path)?;
            }
        }
    }
    let mut loaded = Vec::new();
    for name in chosen {
        let from = library.join(name);
        if !from.is_dir() {
            continue;
        }
        let to = dir.join(name);
        if to.exists() && !is_ours(&to) {
            warn!("an agent has a plugin folder of its own with the name of one it was given");
            continue;
        }
        std::fs::create_dir_all(&dir)?;
        // One plugin that cannot be copied does not keep the agent from
        // starting with the others.
        match copy_plugin(&from, &to) {
            Ok(()) => loaded.push(to),
            Err(err) => warn!("could not give an agent a plugin: {err}"),
        }
    }
    Ok(loaded)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, path: &str, text: &str) {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        std::fs::write(path, text).expect("write");
    }

    fn plugin(dir: &Path) {
        write(
            dir,
            ".claude-plugin/plugin.json",
            r#"{"name":"kit","mcpServers":{"api":{}},"channels":[{"server":"api"}]}"#,
        );
        write(dir, "skills/review/SKILL.md", "review v1");
        write(dir, "refs/note.md", "note");
        write(dir, ".mcp.json", "{}");
        write(dir, ".in_use/lock", "session");
        write(dir, ".git/HEAD", "ref");
    }

    #[test]
    fn a_copy_leaves_the_servers_out_and_is_made_again_only_when_the_plugin_changed() {
        let root = tempfile::tempdir().expect("dir");
        let (from, to) = (root.path().join("kit"), root.path().join("library/kit"));
        plugin(&from);
        std::fs::create_dir_all(to.parent().expect("parent")).expect("library");
        copy_plugin(&from, &to).expect("copy");
        assert!(is_ours(&to));
        assert!(to.join("refs/note.md").is_file());
        for gone in [".mcp.json", ".in_use", ".git"] {
            assert!(!to.join(gone).exists(), "{gone}");
        }
        let manifest =
            std::fs::read_to_string(to.join(".claude-plugin/plugin.json")).expect("json");
        assert!(
            manifest.contains("\"kit\"") && !manifest.contains("mcpServers"),
            "{manifest}"
        );
        assert!(!manifest.contains("channels"), "{manifest}");

        // Unchanged, the copy stays as it is: a file put in it survives.
        write(&to, "left.txt", "here");
        copy_plugin(&from, &to).expect("again");
        assert!(to.join("left.txt").is_file());
        // A change in the plugin makes a new copy.
        write(&from, "skills/review/SKILL.md", "review v2, longer");
        copy_plugin(&from, &to).expect("changed");
        assert!(!to.join("left.txt").exists());
        let skill = std::fs::read_to_string(to.join("skills/review/SKILL.md")).expect("skill");
        assert_eq!(skill, "review v2, longer");
    }

    #[test]
    fn the_agent_gets_marked_copies_and_keeps_its_own_folders() {
        let root = tempfile::tempdir().expect("dir");
        let library = root.path().join("library");
        let workspace = root.path().join("bot");
        plugin(&library.join("kit"));
        plugin(&library.join("data"));
        // A plugin folder the agent made itself, with a name it was given.
        write(&workspace, ".claude/plugins/data/own.md", "its own");
        let loaded = sync(&library, &workspace, &["data".into(), "kit".into()]).expect("sync");
        let kit = workspace.join(".claude/plugins/kit");
        assert_eq!(loaded, std::slice::from_ref(&kit));
        assert!(kit.join("skills/review/SKILL.md").is_file());
        assert!(workspace.join(".claude/plugins/data/own.md").is_file());

        assert!(sync(&library, &workspace, &[]).expect("none").is_empty());
        assert!(!kit.exists());
        assert!(workspace.join(".claude/plugins/data").is_dir());
    }
}
