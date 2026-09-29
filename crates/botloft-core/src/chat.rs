//! How chat items read (spec 8.2): one-line summaries of tool calls, text
//! cut to size, and the line the conversation list shows.

use serde_json::Value;

use crate::protocol::{ActivityKind, ChatBody, SenderKind};

/// Longest tool input kept, in bytes.
pub const TOOL_INPUT_MAX: usize = 4 * 1024;
/// Longest plan kept, in bytes: the owner reads all of it to approve it.
pub const PLAN_INPUT_MAX: usize = 32 * 1024;
/// The tool Claude Code uses to leave plan mode with a plan (spec 10.1).
pub const PLAN_TOOL: &str = "ExitPlanMode";
/// Longest tool output kept, in bytes.
pub const TOOL_OUTPUT_MAX: usize = 8 * 1024;
/// Longest activity line, in characters.
pub const ACTIVITY_MAX_CHARS: usize = 120;
/// Longest summary, in characters.
const SUMMARY_MAX_CHARS: usize = 160;

/// `text` cut to at most `max` bytes on a character boundary, with an
/// ellipsis when something was cut.
pub fn clip(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_owned();
    }
    let mut end = max.saturating_sub('…'.len_utf8());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &text[..end])
}

/// `text` on one line, at most `max` characters.
pub fn one_line(text: &str, max: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        return flat;
    }
    let cut: String = flat.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}

/// Longest input kept for `tool`, in bytes.
pub fn tool_input_max(tool: &str) -> usize {
    if tool == PLAN_TOOL {
        PLAN_INPUT_MAX
    } else {
        TOOL_INPUT_MAX
    }
}

fn field<'a>(input: &'a Value, name: &str) -> Option<&'a str> {
    input.get(name).and_then(Value::as_str)
}

/// A one-line description of a tool call, from its input.
pub fn tool_summary(name: &str, input: &Value) -> String {
    let path = || {
        field(input, "file_path")
            .or_else(|| field(input, "notebook_path"))
            .or_else(|| field(input, "path"))
            .map(file_name)
    };
    let summary = match name {
        "Bash" | "PowerShell" => field(input, "description")
            .map(str::to_owned)
            .or_else(|| field(input, "command").map(str::to_owned)),
        "Read" | "Edit" | "MultiEdit" | "Write" | "NotebookEdit" => path(),
        "Grep" | "Glob" => field(input, "pattern").map(str::to_owned),
        "WebFetch" => field(input, "url").map(str::to_owned),
        "WebSearch" => field(input, "query").map(str::to_owned),
        "Task" | "Agent" => field(input, "description").map(str::to_owned),
        "TodoWrite" => Some("Updated the plan".to_owned()),
        // The plan's first line, usually its title.
        PLAN_TOOL => field(input, "plan").and_then(|plan| {
            plan.lines()
                .map(|line| line.trim_start_matches('#').trim())
                .find(|line| !line.is_empty())
                .map(str::to_owned)
        }),
        // Claude Code loads deferred tools (such as ours) by name first.
        "ToolSearch" => field(input, "query").map(|query| match query.strip_prefix("select:") {
            Some(names) => {
                let labels: Vec<_> = names.split(',').map(|n| tool_label(n.trim())).collect();
                format!("load {}", labels.join(", "))
            }
            None => query.to_owned(),
        }),
        "mcp__botloft__send_message" => {
            field(input, "to").map(|to| format!("to @{}", to.trim_start_matches('@')))
        }
        "mcp__botloft__complete_task" => field(input, "task_id").map(|id| format!("task {id}")),
        _ => None,
    };
    one_line(&summary.unwrap_or_default(), SUMMARY_MAX_CHARS)
}

/// The last part of a path, whichever slash it uses.
fn file_name(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .find(|part| !part.is_empty())
        .unwrap_or(path)
        .to_owned()
}

/// A readable name for a tool: `mcp__botloft__send_message` reads
/// `send_message`.
pub fn tool_label(name: &str) -> &str {
    name.rsplit("__").next().unwrap_or(name)
}

/// The conversation-list line for a chat item, and what kind of line it
/// is; `None` for items that do not change it (turn ends). The text has no
/// wording of its own ("You:", "Waiting for approval"): the app adds it in
/// the owner's language.
pub fn activity_line(body: &ChatBody) -> Option<(ActivityKind, String)> {
    let (kind, text) = match body {
        ChatBody::Inbound(item) => match item.message.from_kind {
            SenderKind::Owner => (ActivityKind::Owner, item.message.body.clone()),
            _ => (ActivityKind::Message, item.message.body.clone()),
        },
        ChatBody::Reply(item) => (ActivityKind::Reply, item.text.clone()),
        ChatBody::Tool(item) => (
            ActivityKind::Tool,
            match item.summary.as_str() {
                "" => tool_label(&item.name).to_owned(),
                summary => format!("{} · {summary}", tool_label(&item.name)),
            },
        ),
        ChatBody::Approval(item) => (
            ActivityKind::Approval,
            tool_label(&item.tool_name).to_owned(),
        ),
        ChatBody::Notice(item) => (ActivityKind::Notice, item.text.clone()),
        ChatBody::Turn(_) => return None,
    };
    Some((kind, one_line(&text, ACTIVITY_MAX_CHARS)))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn clip_keeps_whole_characters() {
        assert_eq!(clip("hello", 10), "hello");
        let cut = clip("olá mundo", 5);
        assert!(cut.len() <= 5, "{cut}");
        assert!(cut.ends_with('…'));
        // Six bytes hold one two-byte letter and the three-byte ellipsis.
        assert_eq!(clip("ééééé", 6), "é…");
    }

    #[test]
    fn summaries_name_what_the_tool_touches() {
        assert_eq!(
            tool_summary(
                "Bash",
                &json!({ "command": "npm test", "description": "Run the tests" })
            ),
            "Run the tests"
        );
        assert_eq!(
            tool_summary("Bash", &json!({ "command": "ls\n-la" })),
            "ls -la"
        );
        assert_eq!(
            tool_summary("Edit", &json!({ "file_path": "C:\\work\\src\\main.rs" })),
            "main.rs"
        );
        assert_eq!(tool_summary("Grep", &json!({ "pattern": "TODO" })), "TODO");
        assert_eq!(
            tool_summary(
                "mcp__botloft__send_message",
                &json!({ "to": "@writer", "body": "x" })
            ),
            "to @writer"
        );
        assert_eq!(
            tool_summary(
                "ToolSearch",
                &json!({ "query": "select:mcp__botloft__send_message,Read", "max_results": 3 })
            ),
            "load send_message, Read"
        );
        assert_eq!(
            tool_summary("ToolSearch", &json!({ "query": "notebook jupyter" })),
            "notebook jupyter"
        );
        assert_eq!(tool_summary("SomethingNew", &json!({ "a": 1 })), "");
        assert_eq!(
            tool_summary(
                PLAN_TOOL,
                &json!({ "plan": "
## Move the cache

1. Read" })
            ),
            "Move the cache"
        );
        assert_eq!(tool_label("mcp__botloft__send_message"), "send_message");
        assert_eq!(tool_label("Bash"), "Bash");
    }

    #[test]
    fn long_lines_are_flattened_and_cut() {
        let text = format!("first line\nsecond {}", "x".repeat(300));
        let line = one_line(&text, 20);
        assert_eq!(line.chars().count(), 20);
        assert!(line.starts_with("first line second"));
        assert!(line.ends_with('…'));
    }
}
