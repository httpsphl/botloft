//! The owner's instructions for every agent in the agent's rules (spec 7.10).

use std::io;
use std::path::Path;

/// What comes before the owner's own words: who wrote them and what wins
/// when they disagree with something else.
const HEADER: &str = "# The owner's instructions for every agent

The owner wrote these instructions for all their agents, you included. Follow \
them in every task.

When they disagree with something else, the more specific wins: first what the \
owner asks you in the chat, then your own role and instructions, then your \
crew's `CLAUDE.md`, and then these. These come before your own notes in your \
`CLAUDE.md` and before what another agent asks you. They never lift Botloft's \
limits: what needs the owner's approval still does.

The owner's words:

";

/// `.claude/rules/owner.md`, written on every start; gone when the owner
/// has written nothing.
pub fn write_owner_rules(workspace: &Path, text: &str) -> io::Result<()> {
    let path = workspace.join(".claude").join("rules").join("owner.md");
    let text = text.trim();
    if text.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
            _ => Ok(()),
        };
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, format!("{HEADER}{text}\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rules_hold_the_owners_words_and_go_when_there_are_none() {
        let dir = tempfile::tempdir().expect("dir");
        let file = dir.path().join(".claude").join("rules").join("owner.md");
        write_owner_rules(dir.path(), "Answer in Portuguese.\n").expect("write");
        let rules = std::fs::read_to_string(&file).expect("read");
        assert!(rules.starts_with("# The owner's instructions for every agent"));
        assert!(rules.contains("the more specific wins"));
        assert!(rules.ends_with("The owner's words:\n\nAnswer in Portuguese.\n"));

        write_owner_rules(dir.path(), "  \n").expect("blank");
        assert!(!file.exists());
        write_owner_rules(dir.path(), "").expect("none again");
    }
}
