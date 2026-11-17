//! What an agent's rules say about the owner's skills (spec 38.6).

use std::io;
use std::path::Path;

/// `.claude/rules/skills.md`: the skills the agent has and the others it
/// can ask for, by name only. Nothing when the owner has none.
pub(super) fn write(workspace: &Path, chosen: &[String], others: &[String]) -> io::Result<()> {
    let path = workspace.join(".claude").join("rules").join("skills.md");
    if chosen.is_empty() && others.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
            _ => Ok(()),
        };
    }
    let mut text = String::from("# The owner's skills\n");
    if !chosen.is_empty() {
        text.push_str(
            "\nThe owner gave you these skills, from their own Claude Code. They are in \
             `.claude/skills`: use them as usual.\n\n",
        );
        for name in chosen {
            text.push_str(&format!("- {name}\n"));
        }
    }
    if !others.is_empty() {
        text.push_str(
            "\nThe owner has these skills too. When one would help with a task, ask the \
             owner for it with ask_skill, by its name:\n\n",
        );
        for name in others {
            text.push_str(&format!("- {name}\n"));
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, text)
}
