//! How `search_history` tells a chat item to the bot (spec 8.8): who wrote
//! it, when, what kind it is and its text.

use botloft_core::cards::as_markdown;
use botloft_core::ids::BotId;
use botloft_core::protocol::{ChatBody, ChatItem, QuestionKind, SenderKind};
use botloft_store::Store;
use jiff::Timestamp;
use jiff::tz::TimeZone;

/// The parts of a chat the bot reads around a hit: what was said, not the
/// work (tools, approvals, notices, ends of turns).
pub(super) fn conversational(item: &ChatItem) -> bool {
    matches!(
        item.body,
        ChatBody::Inbound(_) | ChatBody::Reply(_) | ChatBody::Question(_) | ChatBody::Card(_)
    )
}

/// What the bot reads as an item's `kind`.
pub(super) fn kind_of(item: &ChatItem) -> &'static str {
    match &item.body {
        ChatBody::Inbound(_) => "message",
        ChatBody::Reply(_) => "reply",
        ChatBody::Question(question) if question.question.kind == QuestionKind::Note => "note",
        ChatBody::Question(_) => "question",
        ChatBody::Card(_) => "card",
        _ => "other",
    }
}

/// An item's whole text: a message with the names of its files, a
/// question with the owner's answer, a card as markdown.
pub(super) fn text_of(item: &ChatItem) -> String {
    match &item.body {
        ChatBody::Inbound(inbound) => {
            let message = &inbound.message;
            if message.attachments.is_empty() {
                return message.body.clone();
            }
            let names: Vec<&str> = message
                .attachments
                .iter()
                .map(|file| file.name.as_str())
                .collect();
            format!("{}\n\n(attached: {})", message.body, names.join(", "))
        }
        ChatBody::Reply(reply) => reply.text.clone(),
        ChatBody::Question(item) => match &item.question.answer {
            Some(answer) => format!("{}\n\nThe owner's answer: {answer}", item.question.text),
            None => item.question.text.clone(),
        },
        ChatBody::Card(card) => as_markdown(card),
        _ => String::new(),
    }
}

/// At most `max` characters of `text`, with a mark when it was cut.
pub(super) fn cut(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((at, _)) => format!("{}… (cut)", &text[..at]),
        None => text.to_owned(),
    }
}

/// Names the writers and times of the bot's chat items.
pub(super) struct Teller<'a> {
    store: &'a Store,
    zone: TimeZone,
}

impl<'a> Teller<'a> {
    pub(super) fn new(store: &'a Store) -> Self {
        Self {
            store,
            zone: TimeZone::system(),
        }
    }

    /// The computer's time zone, as the bot reads it once per answer.
    pub(super) fn zone(&self) -> String {
        self.zone.iana_name().unwrap_or("UTC").to_owned()
    }

    /// `at` in the computer's time zone, like `2026-11-10 14:05`.
    pub(super) fn at(&self, at: i64) -> String {
        Timestamp::from_millisecond(at)
            .map(|ts| {
                ts.to_zoned(self.zone.clone())
                    .strftime("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_default()
    }

    /// Who wrote the item: `owner`, `Botloft`, another bot's `@handle`
    /// (`a deleted bot` once it is gone) or `you`.
    pub(super) fn from(&self, item: &ChatItem) -> String {
        let ChatBody::Inbound(inbound) = &item.body else {
            return "you".to_owned();
        };
        let message = &inbound.message;
        match message.from_kind {
            SenderKind::Owner => "owner".to_owned(),
            SenderKind::System => "Botloft".to_owned(),
            SenderKind::Bot => message
                .from_bot_id
                .as_ref()
                .and_then(|id| self.handle(id))
                .map_or_else(|| "a deleted bot".to_owned(), |handle| format!("@{handle}")),
        }
    }

    fn handle(&self, id: &BotId) -> Option<String> {
        self.store.bot(id).ok().flatten().map(|bot| bot.handle)
    }
}

#[cfg(test)]
mod tests {
    use super::cut;

    #[test]
    fn cuts_by_characters_and_says_so() {
        assert_eq!(cut("ação", 10), "ação");
        assert_eq!(cut("ação rápida", 4), "ação… (cut)");
    }
}
