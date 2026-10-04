//! How chat items read (spec 8.2): one-line summaries of tool calls, text
//! cut to size, and the line the conversation list shows.

use serde_json::Value;

use crate::command::{COMMAND_INPUT_MAX, is_command};
use crate::protocol::{Activity, ActivityKind, ChatBody, NoticeCode, SenderKind};

/// Longest tool input kept, in bytes.
pub const TOOL_INPUT_MAX: usize = 4 * 1024;
/// Longest plan kept, in bytes: the owner reads all of it to approve it.
pub const PLAN_INPUT_MAX: usize = 32 * 1024;
/// The tool Claude Code uses to leave plan mode with a plan (spec 10.1).
pub const PLAN_TOOL: &str = "ExitPlanMode";
/// The chief's tool to suggest a new bot (spec 10.2).
pub const SUGGEST_TOOL: &str = "mcp__botloft__suggest_bot";
/// A bot asking the owner for a routine (spec 20.12).
pub const ROUTINE_TOOL: &str = "mcp__botloft__schedule_routine";
/// A bot asking the owner to change one of its routines (spec 20.12).
pub const CHANGE_ROUTINE_TOOL: &str = "mcp__botloft__change_routine";
/// A bot asking the owner to delete one of its routines (spec 20.12).
pub const DELETE_ROUTINE_TOOL: &str = "mcp__botloft__delete_routine";
/// A bot asking to use its browser on a site (spec 21.5). Not a tool the
/// model calls: the daemon opens this request from inside the `browser_*`
/// tools.
pub const BROWSER_SITE_TOOL: &str = "mcp__botloft__browser";
/// A bot asking to see or use an app on the owner's desktop (spec 24.2),
/// opened from inside the `desktop_*` tools.
pub const DESKTOP_TOOL: &str = "mcp__botloft__desktop";
/// A bot asking the owner to do something in its browser themselves (spec
/// 21.10), opened from inside `browser_ask_owner`.
pub const BROWSER_HELP_TOOL: &str = "mcp__botloft__browser_help";
/// Longest bot or routine suggestion kept, in bytes: the owner reads and
/// edits all of it.
pub const SUGGESTION_INPUT_MAX: usize = 64 * 1024;
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
    match tool {
        PLAN_TOOL => PLAN_INPUT_MAX,
        SUGGEST_TOOL | ROUTINE_TOOL | CHANGE_ROUTINE_TOOL => SUGGESTION_INPUT_MAX,
        tool if is_command(tool) => COMMAND_INPUT_MAX,
        _ => TOOL_INPUT_MAX,
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
        // The command itself; what the bot says about it is the explanation.
        "Bash" | "PowerShell" => field(input, "command").map(str::to_owned),
        "Read" | "Edit" | "MultiEdit" | "Write" | "NotebookEdit" => path(),
        "Grep" | "Glob" => field(input, "pattern").map(str::to_owned),
        "WebFetch" => field(input, "url").map(str::to_owned),
        "WebSearch" => field(input, "query").map(str::to_owned),
        "Task" | "Agent" => field(input, "description").map(str::to_owned),
        // The app names these; nothing to add in any language.
        "TodoWrite" | "mcp__botloft__complete_task" => None,
        // The plan's first line, usually its title.
        PLAN_TOOL => field(input, "plan").and_then(|plan| {
            plan.lines()
                .map(|line| line.trim_start_matches('#').trim())
                .find(|line| !line.is_empty())
                .map(str::to_owned)
        }),
        // Claude Code loads deferred tools (such as ours) by name first; the
        // app names the tools loaded, from the input.
        "ToolSearch" => field(input, "query")
            .filter(|query| !query.starts_with("select:"))
            .map(str::to_owned),
        "mcp__botloft__send_message" => {
            field(input, "to").map(|to| format!("@{}", to.trim_start_matches('@')))
        }
        SUGGEST_TOOL | ROUTINE_TOOL | CHANGE_ROUTINE_TOOL | DELETE_ROUTINE_TOOL => {
            field(input, "name").map(str::to_owned)
        }
        BROWSER_SITE_TOOL => field(input, "site").map(str::to_owned),
        DESKTOP_TOOL => field(input, "app").map(str::to_owned),
        "mcp__botloft__desktop_look" | "mcp__botloft__desktop_screenshot" => {
            field(input, "why").map(str::to_owned)
        }
        BROWSER_HELP_TOOL | "mcp__botloft__browser_ask_owner" => {
            field(input, "task").map(str::to_owned)
        }
        "mcp__botloft__browser_open" => field(input, "url").map(str::to_owned),
        "mcp__botloft__share_file" => input.get("files").and_then(Value::as_array).map(|files| {
            let names: Vec<String> = files
                .iter()
                .filter_map(Value::as_str)
                .map(file_name)
                .collect();
            names.join(", ")
        }),
        // Element refs ("e12") mean nothing to the owner, and a direction
        // needs words: the app says those (spec 21.8).
        "mcp__botloft__browser_press" => field(input, "key").map(str::to_owned),
        _ => None,
    };
    one_line(&summary.unwrap_or_default(), SUMMARY_MAX_CHARS)
}

/// The full path of the file a `Write`/`Edit`/`NotebookEdit` call changes.
pub fn tool_file(name: &str, input: &Value) -> Option<String> {
    matches!(name, "Edit" | "MultiEdit" | "Write" | "NotebookEdit")
        .then(|| field(input, "file_path").or_else(|| field(input, "notebook_path")))
        .flatten()
        .filter(|path| !path.is_empty() && path.len() <= 1024)
        .map(str::to_owned)
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

/// The conversation-list line for a chat item, from `at`; `None` for
/// items that do not change it (turn ends). It has no wording of its own
/// ("You:", "Waiting for approval", a tool's name): the app adds it in the
/// owner's language, from `kind` and `tool`.
pub fn activity(body: &ChatBody, at: i64) -> Option<Activity> {
    let (kind, text) = match body {
        ChatBody::Inbound(item) => match item.message.from_kind {
            SenderKind::Owner => (ActivityKind::Owner, item.message.body.clone()),
            _ => (ActivityKind::Message, item.message.body.clone()),
        },
        ChatBody::Reply(item) => (ActivityKind::Reply, item.text.clone()),
        // A command reads better in the bot's words, when it gave them.
        ChatBody::Tool(item) => (
            ActivityKind::Tool,
            item.explanation.as_ref().unwrap_or(&item.summary).clone(),
        ),
        ChatBody::Approval(item) => (
            ActivityKind::Approval,
            item.explanation.as_ref().unwrap_or(&item.summary).clone(),
        ),
        ChatBody::Question(item) => (ActivityKind::Question, item.question.text.clone()),
        // A compaction is housekeeping, not news about the conversation.
        ChatBody::Notice(item)
            if matches!(
                item.code,
                Some(NoticeCode::Compacted | NoticeCode::AutoCompacted | NoticeCode::CompactFailed)
            ) =>
        {
            return None;
        }
        ChatBody::Notice(item) => (ActivityKind::Notice, item.text.clone()),
        ChatBody::Turn(_) => return None,
    };
    let tool = match body {
        ChatBody::Tool(item) => Some(item.name.clone()),
        ChatBody::Approval(item) => Some(item.tool_name.clone()),
        _ => None,
    };
    Some(Activity {
        kind,
        text: one_line(&text, ACTIVITY_MAX_CHARS),
        tool,
        at,
    })
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
            "npm test"
        );
        assert_eq!(
            tool_summary("PowerShell", &json!({ "command": "ls\n-la" })),
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
            "@writer"
        );
        assert_eq!(
            tool_summary(
                "ToolSearch",
                &json!({ "query": "select:mcp__botloft__send_message,Read", "max_results": 3 })
            ),
            ""
        );
        assert_eq!(tool_summary("TodoWrite", &json!({ "todos": [] })), "");
        assert_eq!(
            tool_summary("ToolSearch", &json!({ "query": "notebook jupyter" })),
            "notebook jupyter"
        );
        assert_eq!(tool_summary("SomethingNew", &json!({ "a": 1 })), "");
        assert_eq!(
            tool_summary(
                "mcp__botloft__share_file",
                &json!({ "files": [r"C:\work\report.pdf", "out/chart.png"] })
            ),
            "report.pdf, chart.png"
        );
        assert_eq!(
            tool_summary(
                "mcp__botloft__browser_open",
                &json!({ "url": "https://example.com/a" })
            ),
            "https://example.com/a"
        );
        assert_eq!(
            tool_summary(
                "mcp__botloft__browser_type",
                &json!({ "ref": "e4", "text": "secret" })
            ),
            ""
        );
        assert_eq!(
            tool_summary(BROWSER_SITE_TOOL, &json!({ "site": "example.com" })),
            "example.com"
        );
        assert_eq!(
            tool_summary(
                BROWSER_HELP_TOOL,
                &json!({ "task": "Sign in to GitHub", "site": "github.com" })
            ),
            "Sign in to GitHub"
        );
        assert_eq!(
            tool_summary(
                PLAN_TOOL,
                &json!({ "plan": "
## Move the cache

1. Read" })
            ),
            "Move the cache"
        );
        assert_eq!(tool_input_max("Bash"), COMMAND_INPUT_MAX);
        assert_eq!(tool_input_max("Read"), TOOL_INPUT_MAX);
        assert_eq!(tool_label("mcp__botloft__send_message"), "send_message");
        assert_eq!(tool_label("Bash"), "Bash");
    }

    #[test]
    fn a_compaction_does_not_change_the_conversation_list_line() {
        use crate::protocol::{NoticeItem, NoticeLevel};
        let notice = |code| {
            ChatBody::Notice(NoticeItem {
                level: NoticeLevel::Info,
                code: Some(code),
                text: "The conversation was compacted.".into(),
            })
        };
        assert_eq!(activity(&notice(NoticeCode::Compacted), 1), None);
        assert_eq!(activity(&notice(NoticeCode::AutoCompacted), 1), None);
        assert_eq!(activity(&notice(NoticeCode::CompactFailed), 1), None);
        let limit = activity(&notice(NoticeCode::UsageLimit), 1).expect("a line");
        assert_eq!(limit.kind, ActivityKind::Notice);
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
