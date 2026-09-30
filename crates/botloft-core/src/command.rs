//! Shell commands a bot runs (`Bash`, `PowerShell`): the one kind of tool
//! call whose input carries the bot's own words about it, which the owner
//! reads before allowing the command (spec 10.1).

use serde_json::Value;

use crate::chat::one_line;

/// Longest command input kept, in bytes: the owner can read all of what
/// they allow, as with a plan.
pub const COMMAND_INPUT_MAX: usize = 32 * 1024;
/// Longest explanation, in characters.
const EXPLANATION_MAX_CHARS: usize = 300;

/// Whether `tool` runs a shell command.
pub fn is_command(tool: &str) -> bool {
    matches!(tool, "Bash" | "PowerShell")
}

/// What the bot says a command is for: the `description` Claude Code's
/// shell tools take next to `command`, on one line. The model writes it and
/// may leave it out, and one that only repeats the command explains nothing.
pub fn tool_explanation(tool: &str, input: &Value) -> Option<String> {
    if !is_command(tool) {
        return None;
    }
    let flat = |name: &str| {
        let text = input.get(name).and_then(Value::as_str)?;
        Some(text.split_whitespace().collect::<Vec<_>>().join(" "))
    };
    let said = flat("description").filter(|said| !said.is_empty())?;
    if flat("command").as_deref() == Some(said.as_str()) {
        return None;
    }
    Some(one_line(&said, EXPLANATION_MAX_CHARS))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_explanation_is_what_the_bot_wrote_about_a_command() {
        let input = json!({ "command": "pip install requests", "description": "Instala o\npacote  requests" });
        assert_eq!(
            tool_explanation("Bash", &input).as_deref(),
            Some("Instala o pacote requests")
        );
        assert_eq!(
            tool_explanation("PowerShell", &input).as_deref(),
            Some("Instala o pacote requests")
        );
    }

    #[test]
    fn a_command_without_one_has_none() {
        assert_eq!(
            tool_explanation("Bash", &json!({ "command": "make" })),
            None
        );
        let blank = json!({ "command": "make", "description": "  \n" });
        assert_eq!(tool_explanation("Bash", &blank), None);
        let not_text = json!({ "command": "make", "description": 3 });
        assert_eq!(tool_explanation("Bash", &not_text), None);
        // Repeating the command is not explaining it.
        let echo = json!({ "command": "npm  run build", "description": "npm run build" });
        assert_eq!(tool_explanation("Bash", &echo), None);
    }

    #[test]
    fn other_tools_have_none() {
        // A helper's `description` is its title, already the summary.
        let helper = json!({ "description": "Find the endpoints", "prompt": "..." });
        assert_eq!(tool_explanation("Agent", &helper), None);
        let file = json!({ "file_path": "a.md", "description": "x" });
        assert_eq!(tool_explanation("Write", &file), None);
    }

    #[test]
    fn a_long_one_is_cut() {
        let input = json!({ "command": "make", "description": "a".repeat(500) });
        let said = tool_explanation("Bash", &input).expect("explanation");
        assert_eq!(said.chars().count(), EXPLANATION_MAX_CHARS);
        assert!(said.ends_with('…'));
    }
}
