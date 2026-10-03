//! What the tests of spec 8.6 share: a bot's context as the app sees it,
//! Claude Code's answers about it and the chat it leaves behind.

use std::time::Duration;

use botloft_core::protocol::{Bot, ChatBody, ContextUsage, NoticeCode};
use botloftd::runtime::fake::FakeProcess;
use botloftd::service::bots;
use botloftd::state::Event;
use serde_json::json;
use tokio::sync::broadcast::Receiver;

use super::supervised::Setup;

pub fn bot(s: &Setup) -> Bot {
    bots::list(&s.daemon, Default::default())
        .expect("list")
        .into_iter()
        .find(|bot| bot.id == s.bot)
        .expect("bot")
}

pub fn shown(s: &Setup) -> ContextUsage {
    bot(s).context.expect("a known context")
}

pub async fn settle() {
    tokio::time::sleep(Duration::from_millis(50)).await;
}

/// Claude Code answering `get_context_usage`, as seen with 2.1.284.
pub async fn holds(process: &FakeProcess, used: u64, window: u64) {
    process
        .answer_control(
            "get_context_usage",
            json!({
                "totalTokens": used, "maxTokens": window, "rawMaxTokens": window,
                "autoCompactThreshold": window - 33_000, "isAutoCompactEnabled": true,
                "percentage": used * 100 / window, "model": "claude-sonnet-5-5",
            }),
        )
        .await;
    settle().await;
}

pub fn asks(process: &FakeProcess) -> usize {
    process
        .control_requests()
        .iter()
        .filter(|subtype| *subtype == "get_context_usage")
        .count()
}

/// The `bot.context` notifications sent so far, oldest first.
pub fn sent(events: &mut Receiver<Event>) -> Vec<Option<ContextUsage>> {
    let mut contexts = Vec::new();
    while let Ok(event) = events.try_recv() {
        if let Event::BotContext(changed) = event {
            contexts.push(changed.context);
        }
    }
    contexts
}

pub fn notices(s: &Setup) -> Vec<(NoticeCode, String)> {
    let items = s
        .daemon
        .store()
        .chat_history(&s.bot, None, 50)
        .expect("history");
    items
        .into_iter()
        .rev()
        .filter_map(|item| match item.body {
            ChatBody::Notice(notice) => Some((notice.code.expect("a code"), notice.text)),
            _ => None,
        })
        .collect()
}

pub fn kinds(s: &Setup) -> Vec<&'static str> {
    let items = s
        .daemon
        .store()
        .chat_history(&s.bot, None, 50)
        .expect("history");
    items
        .iter()
        .rev()
        .map(|item| match item.body {
            ChatBody::Inbound(_) => "inbound",
            ChatBody::Reply(_) => "reply",
            ChatBody::Tool(_) => "tool",
            ChatBody::Approval(_) => "approval",
            ChatBody::Question(_) => "question",
            ChatBody::Turn(_) => "turn",
            ChatBody::Notice(_) => "notice",
        })
        .collect()
}
