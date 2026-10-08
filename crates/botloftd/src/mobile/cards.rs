//! What the phone is shown (spec 28.5): a card for each request and question
//! that waits, with only what the owner needs to decide.

use botloft_core::chat::PLAN_TOOL;
use botloft_core::ids::ApprovalId;
use botloft_core::protocol::{
    ApprovalCard, ApprovalItem, ApprovalStatus, CARD_TEXT_MAX, CardBot, ChatBody, ChatItem,
    Question, QuestionCard, QuestionStatus,
};
use serde_json::Value;
use tracing::warn;

use crate::state::Daemon;

/// The tools the phone may allow: the ones Claude Code asks about in a
/// plain way, which change nothing that lasts. Everything else (a bot or
/// routine suggestion, reaching another crew, the desktop, the browser in
/// hand, connected tools) is answered at the computer, and the phone may
/// only deny it.
pub fn may_allow(tool: &str) -> bool {
    matches!(
        tool,
        "Bash"
            | "PowerShell"
            | "WebFetch"
            | "WebSearch"
            | "Read"
            | "Write"
            | "Edit"
            | "MultiEdit"
            | "NotebookEdit"
    ) || tool == PLAN_TOOL
}

/// The text to read before allowing, and whether it is not all of the
/// request (cut by the daemon when saved, or too long for a phone).
fn text_of(tool: &str, input: &str) -> (String, bool) {
    let parsed: Option<Value> = serde_json::from_str(input).ok();
    let field = |name: &str| {
        parsed
            .as_ref()
            .and_then(|value| value.get(name))
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    let (text, whole) = match tool {
        "Bash" | "PowerShell" => (field("command"), parsed.is_some()),
        PLAN_TOOL => (field("plan"), parsed.is_some()),
        _ => (None, parsed.is_some()),
    };
    // A request the daemon cut short (it does not parse) is shown as it is
    // and marked, and so is one the field of which is missing.
    let text = text.unwrap_or_else(|| input.to_owned());
    if text.len() <= CARD_TEXT_MAX {
        return (text, !whole);
    }
    let mut end = CARD_TEXT_MAX;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (text[..end].to_owned(), true)
}

fn bot_and_crew(daemon: &Daemon, bot: &botloft_core::ids::BotId) -> Option<(CardBot, String)> {
    let store = daemon.store();
    let record = match store.bot(bot) {
        Ok(Some(record)) if record.archived_at.is_none() => record,
        Ok(_) => return None,
        Err(err) => {
            warn!(bot = %bot, "could not read a bot for a card: {err}");
            return None;
        }
    };
    let crew = store.crew(&record.crew_id).ok().flatten()?;
    Some((
        CardBot {
            name: record.name,
            color: record.color,
        },
        crew.name,
    ))
}

/// The card of a request in the chat; `None` for a bot that is gone.
pub fn approval_card(
    daemon: &Daemon,
    item: &ChatItem,
    shown: &ApprovalItem,
) -> Option<ApprovalCard> {
    let (bot, crew) = bot_and_crew(daemon, &item.bot_id)?;
    let (text, cut) = text_of(&shown.tool_name, &shown.input);
    Some(ApprovalCard {
        approval_id: shown.approval_id.clone(),
        bot,
        crew,
        created_at: item.created_at,
        tool_name: shown.tool_name.clone(),
        summary: shown.summary.clone(),
        explanation: shown.explanation.clone(),
        text,
        cut,
        at_computer: !may_allow(&shown.tool_name),
    })
}

pub fn question_card(daemon: &Daemon, question: &Question) -> Option<QuestionCard> {
    let (bot, crew) = bot_and_crew(daemon, &question.bot_id)?;
    Some(QuestionCard {
        question_id: question.id.clone(),
        bot,
        crew,
        created_at: question.created_at,
        text: question.text.clone(),
        options: question.options.clone(),
    })
}

/// The request in the chat, whatever its state: `(item, what it shows)`.
pub fn approval_item(daemon: &Daemon, id: &ApprovalId) -> Option<(ChatItem, ApprovalItem)> {
    let store = daemon.store();
    let record = store.approval(id).ok().flatten()?;
    let item = store.chat_item(&record.chat_item_id).ok().flatten()?;
    match item.body.clone() {
        ChatBody::Approval(shown) => Some((item, shown)),
        _ => None,
    }
}

/// Every request and question waiting right now.
pub fn waiting(daemon: &Daemon) -> (Vec<ApprovalCard>, Vec<QuestionCard>) {
    let mut approvals = Vec::new();
    let pending: Vec<ApprovalId> = {
        let store = daemon.store();
        let bots = store.bots(None, false).unwrap_or_default();
        bots.iter()
            .flat_map(|bot| store.pending_approvals(&bot.id).unwrap_or_default())
            .collect()
    };
    for id in pending {
        if let Some((item, shown)) = approval_item(daemon, &id)
            && shown.status == ApprovalStatus::Pending
            && let Some(card) = approval_card(daemon, &item, &shown)
        {
            approvals.push(card);
        }
    }
    approvals.sort_by_key(|card| card.created_at);
    let open = daemon
        .store()
        .questions(QuestionStatus::Open)
        .unwrap_or_default();
    let mut questions: Vec<QuestionCard> = open
        .iter()
        .filter_map(|question| question_card(daemon, question))
        .collect();
    questions.sort_by_key(|card| card.created_at);
    (approvals, questions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_phone_allows_plain_tools_and_leaves_the_rest_to_the_computer() {
        for tool in ["Bash", "PowerShell", "WebFetch", "Read", "Edit", PLAN_TOOL] {
            assert!(may_allow(tool), "{tool}");
        }
        for tool in [
            "mcp__botloft__suggest_bot",
            "mcp__botloft__ask_crew_access",
            "mcp__botloft__desktop",
            "mcp__botloft__browser",
            "mcp__github__create_issue",
            "Agent",
            "",
        ] {
            assert!(!may_allow(tool), "{tool}");
        }
    }

    #[test]
    fn a_command_is_shown_whole_and_a_cut_or_long_one_is_marked() {
        let input = json!({ "command": "ls -la", "description": "list" }).to_string();
        assert_eq!(text_of("Bash", &input), ("ls -la".to_owned(), false));
        let plan = json!({ "plan": "# Plan\n1. one" }).to_string();
        assert_eq!(
            text_of(PLAN_TOOL, &plan),
            ("# Plan\n1. one".to_owned(), false)
        );
        // Another tool shows its input as it is.
        let read = json!({ "file_path": "a.txt" }).to_string();
        assert_eq!(text_of("Read", &read), (read.clone(), false));

        // What the daemon cut when it saved the request does not parse.
        let cut = format!("{{\"command\": \"{}…", "x".repeat(50));
        let (text, marked) = text_of("Bash", &cut);
        assert_eq!(text, cut);
        assert!(marked);

        // A command over the phone's limit is cut on a character boundary.
        let long = json!({ "command": "é".repeat(CARD_TEXT_MAX) }).to_string();
        let (text, marked) = text_of("Bash", &long);
        assert!(marked && text.len() <= CARD_TEXT_MAX && text.chars().all(|c| c == 'é'));
    }
}
