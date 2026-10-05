//! "Allow always" (spec 10.1): which requests the owner can allow for good,
//! and whether a rule of the bot already allows one.

use botloft_core::ids::BotId;
use botloft_core::protocol::{AllowKind, AllowScope, BotRules, ChatBody};
use botloft_store::ApprovalRecord;
use serde_json::Value;
use tracing::warn;

use crate::browser::sites;
use crate::state::{Daemon, Event};

/// Longest command a rule keeps, in characters: a longer one is a script,
/// worth reading each time.
const COMMAND_MAX: usize = 2000;

/// What "Allow always" would cover for this request; `None` when it cannot
/// be allowed for good.
pub(crate) fn scope_of(tool: &str, input: &Value) -> Option<AllowScope> {
    let field = |name: &str| {
        input
            .get(name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    };
    let (kind, value) = match tool {
        "Bash" | "PowerShell" => (
            AllowKind::Command,
            field("command")
                .filter(|command| command.chars().count() <= COMMAND_MAX)?
                .to_owned(),
        ),
        "WebFetch" => (AllowKind::Site, sites::site_of(field("url")?)?),
        "Read" | "Write" | "Edit" | "MultiEdit" => {
            (AllowKind::File, field("file_path")?.to_owned())
        }
        "NotebookEdit" => (AllowKind::File, field("notebook_path")?.to_owned()),
        "WebSearch" => (AllowKind::Tool, String::new()),
        // One tool of one connected server (spec 25.4); the tools of
        // Botloft itself never ask.
        tool if tool.starts_with("mcp__") && !tool.starts_with("mcp__botloft__") => {
            (AllowKind::Tool, tool.to_owned())
        }
        _ => return None,
    };
    Some(AllowScope {
        tool_name: tool.to_owned(),
        kind,
        value,
    })
}

/// Whether a rule of `bot` allows this request.
pub(crate) fn allowed(daemon: &Daemon, bot: &BotId, tool: &str, input: &Value) -> bool {
    let Some(asked) = scope_of(tool, input) else {
        return false;
    };
    match daemon.store().allow_rules(bot) {
        Ok(rules) => rules.iter().any(|rule| covers(&rule.scope, &asked)),
        Err(err) => {
            warn!(bot = %bot, "could not read the bot's allow rules: {err}");
            false
        }
    }
}

/// Keeps what the allowed request `record` covers as a rule of its bot,
/// when it can be allowed for good. The scope is read from the chat item:
/// the stored input may be cut short.
pub(super) fn remember(daemon: &Daemon, record: &ApprovalRecord) {
    let bot = &record.approval.bot_id;
    let store = daemon.store();
    let scope = match store.chat_item(&record.chat_item_id) {
        Ok(Some(item)) => match item.body {
            ChatBody::Approval(shown) => shown.always,
            _ => None,
        },
        _ => None,
    };
    let Some(scope) = scope else {
        return;
    };
    let rules = store
        .add_allow_rule(bot, &scope, daemon.clock.now_ms())
        .and_then(|_| store.allow_rules(bot));
    drop(store);
    match rules {
        Ok(rules) => daemon.emit(Event::BotRules(BotRules {
            bot_id: bot.clone(),
            rules,
        })),
        Err(err) => warn!(bot = %bot, "could not keep an allow rule: {err}"),
    }
}

/// Whether a rule for `rule` allows a request for `asked`. A site also
/// covers its subdomains, as in the browser (spec 21.5).
fn covers(rule: &AllowScope, asked: &AllowScope) -> bool {
    rule.tool_name == asked.tool_name
        && rule.kind == asked.kind
        && match rule.kind {
            AllowKind::Site => sites::covers(&rule.value, &asked.value),
            _ => rule.value == asked.value,
        }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_tool_of_a_connected_server_is_allowed_one_tool_at_a_time() {
        let scope = scope_of("mcp__linkedin__search_people", &json!({ "q": "x" }));
        assert_eq!(
            scope.map(|scope| (scope.kind, scope.value)),
            Some((AllowKind::Tool, "mcp__linkedin__search_people".to_owned()))
        );
        let rule = |tool: &str| AllowScope {
            tool_name: tool.into(),
            kind: AllowKind::Tool,
            value: tool.into(),
        };
        assert!(!covers(
            &rule("mcp__linkedin__search_people"),
            &rule("mcp__linkedin__send_message")
        ));
        // Botloft's own tools never ask, so there is nothing to allow.
        assert_eq!(scope_of("mcp__botloft__send_message", &json!({})), None);
    }

    #[test]
    fn commands_sites_files_and_searches_can_be_allowed_for_good() {
        let command = scope_of(
            "Bash",
            &json!({ "command": " git status ", "description": "x" }),
        );
        assert_eq!(
            command.map(|scope| (scope.kind, scope.value)),
            Some((AllowKind::Command, "git status".to_owned()))
        );
        let site = scope_of("WebFetch", &json!({ "url": "https://www.Example.com/a?b" }));
        assert_eq!(
            site.map(|scope| scope.value),
            Some("example.com".to_owned())
        );
        let file = scope_of("NotebookEdit", &json!({ "notebook_path": r"C:\w\a.ipynb" }));
        assert_eq!(file.map(|scope| scope.kind), Some(AllowKind::File));
        let search = scope_of("WebSearch", &json!({ "query": "rust" }));
        assert_eq!(search.map(|scope| scope.value), Some(String::new()));
    }

    #[test]
    fn plans_suggestions_long_scripts_and_unknown_tools_cannot() {
        assert_eq!(scope_of("ExitPlanMode", &json!({ "plan": "# Plan" })), None);
        assert_eq!(
            scope_of("mcp__botloft__suggest_bot", &json!({ "name": "A" })),
            None
        );
        assert_eq!(scope_of("Glob", &json!({ "pattern": "*" })), None);
        assert_eq!(scope_of("Bash", &json!({ "command": "  " })), None);
        let script = "echo 1\n".repeat(400);
        assert_eq!(scope_of("Bash", &json!({ "command": script })), None);
        assert_eq!(
            scope_of("WebFetch", &json!({ "url": "file:///C:/a" })),
            None
        );
    }

    #[test]
    fn a_rule_covers_the_same_request_and_a_site_its_subdomains() {
        let rule = |tool: &str, kind, value: &str| AllowScope {
            tool_name: tool.into(),
            kind,
            value: value.into(),
        };
        let git = rule("Bash", AllowKind::Command, "git status");
        assert!(covers(&git, &git));
        assert!(!covers(
            &git,
            &rule("Bash", AllowKind::Command, "git status --short")
        ));
        assert!(!covers(
            &git,
            &rule("PowerShell", AllowKind::Command, "git status")
        ));
        let site = rule("WebFetch", AllowKind::Site, "example.com");
        assert!(covers(
            &site,
            &rule("WebFetch", AllowKind::Site, "docs.example.com")
        ));
        assert!(!covers(
            &site,
            &rule("WebFetch", AllowKind::Site, "badexample.com")
        ));
    }
}
