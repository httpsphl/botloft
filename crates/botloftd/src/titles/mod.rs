//! Titles made by AI (spec 40.6). A conversation that is not the main one
//! is called by the start of the owner's first message until its first
//! reply; then Claude Code is asked, once, for a few words that name it. A
//! title the owner gave is never changed, and one that could not be made
//! leaves the start of the message.

mod claude;

use std::collections::HashSet;
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};

use botloft_core::ids::ConversationId;
use botloft_core::protocol::{ChatBody, ChatItem, SenderKind};
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, warn};

pub use self::claude::ClaudeTitles;
use crate::service::conversations::changed;
use crate::state::{Daemon, Event};

/// The longest title kept, in characters, as the owner's own (`validate::name`).
const MAX_CHARS: usize = botloft_core::validate::NAME_MAX_CHARS;
/// Words in a title made by AI.
const MAX_WORDS: usize = 6;
/// Characters of the first message that name the conversation meanwhile.
const PROVISIONAL_CHARS: usize = 40;
/// What the AI reads of the first message and of the reply.
pub const FIRST_CHARS: usize = 1_000;
pub const REPLY_CHARS: usize = 500;

/// What a title is made from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleAsk {
    pub first: String,
    pub reply: String,
}

/// Makes a title; tests put a fake one in place of Claude Code.
pub trait TitleMaker: Send + Sync {
    fn make(&self, daemon: &Daemon, ask: &TitleAsk) -> io::Result<String>;
}

#[derive(Default)]
pub struct Titles {
    maker: Mutex<Option<Arc<dyn TitleMaker>>>,
    /// The conversations asked for since the daemon started: each once.
    asked: Mutex<HashSet<ConversationId>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Titles {
    pub fn set_maker(&self, maker: Arc<dyn TitleMaker>) {
        *lock(&self.maker) = Some(maker);
    }

    fn maker(&self) -> Arc<dyn TitleMaker> {
        lock(&self.maker)
            .clone()
            .unwrap_or_else(|| Arc::new(ClaudeTitles))
    }

    /// True the first time `conversation` is asked for.
    fn first_ask(&self, conversation: &ConversationId) -> bool {
        lock(&self.asked).insert(conversation.clone())
    }
}

/// Follows what goes into the chats of conversations that are not the main
/// one. Runs as long as the daemon.
pub async fn run(daemon: Arc<Daemon>) {
    let mut events = daemon.subscribe();
    loop {
        match events.recv().await {
            Ok(Event::ChatItem(changed)) => noticed(&daemon, &changed.item),
            Ok(_) | Err(RecvError::Lagged(_)) => {}
            Err(RecvError::Closed) => return,
        }
    }
}

fn noticed(daemon: &Arc<Daemon>, item: &ChatItem) {
    if item.conversation_id.is_main_of(&item.bot_id) {
        return;
    }
    match &item.body {
        ChatBody::Inbound(inbound) if inbound.message.from_kind == SenderKind::Owner => {
            provisional(daemon, &item.conversation_id, &inbound.message.body);
        }
        ChatBody::Reply(_) => ask(daemon, &item.conversation_id),
        _ => {}
    }
}

/// The first message names a conversation that has no name yet.
fn provisional(daemon: &Daemon, id: &ConversationId, text: &str) {
    let Some(title) = start_of(text) else {
        return;
    };
    {
        let store = daemon.store();
        let Ok(Some(record)) = store.conversation(id) else {
            return;
        };
        if record.title.is_some() || record.title_by.is_some() {
            return;
        }
        if let Err(err) = store.set_conversation_title(id, Some(&title), None) {
            warn!("could not name a conversation by its first message: {err}");
            return;
        }
    }
    changed(daemon, id);
}

/// After the first reply, Claude Code names the conversation once.
fn ask(daemon: &Arc<Daemon>, id: &ConversationId) {
    let ask = {
        let store = daemon.store();
        let Ok(Some(record)) = store.conversation(id) else {
            return;
        };
        if record.title_by.is_some() || !daemon.titles.first_ask(id) {
            return;
        }
        let Ok(mut items) = store.chat_history_in(id, None, 50) else {
            return;
        };
        items.reverse();
        let first = items.iter().find_map(|item| match &item.body {
            ChatBody::Inbound(inbound) if inbound.message.from_kind == SenderKind::Owner => {
                Some(clip(&inbound.message.body, FIRST_CHARS))
            }
            _ => None,
        });
        let reply = items.iter().find_map(|item| match &item.body {
            ChatBody::Reply(reply) => Some(clip(&reply.text, REPLY_CHARS)),
            _ => None,
        });
        match (first, reply) {
            (Some(first), Some(reply)) => TitleAsk { first, reply },
            // Nothing the owner wrote: the start of a message stays empty.
            _ => return,
        }
    };
    let daemon = Arc::clone(daemon);
    let id = id.clone();
    tokio::task::spawn_blocking(move || {
        let made = daemon.titles.maker().make(&daemon, &ask);
        // The text is the owner's: never in the log (spec 40.6).
        match made.map(|raw| clean(&raw)) {
            Ok(Some(title)) => named(&daemon, &id, &title),
            Ok(None) => debug!("the title made for a conversation was empty"),
            Err(err) => debug!("no title for a conversation: {err}"),
        }
    });
}

/// The AI's title, unless the owner named it meanwhile.
fn named(daemon: &Daemon, id: &ConversationId, title: &str) {
    {
        let store = daemon.store();
        let Ok(Some(record)) = store.conversation(id) else {
            return;
        };
        if record.title_by.is_some() {
            return;
        }
        if let Err(err) = store.set_conversation_title(id, Some(title), Some("ai")) {
            warn!("could not keep the title of a conversation: {err}");
            return;
        }
    }
    changed(daemon, id);
}

/// At most `max` characters of `text`.
fn clip(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

/// The start of the first message, on one line, cut at a word.
fn start_of(text: &str) -> Option<String> {
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.is_empty() {
        return None;
    }
    if line.chars().count() <= PROVISIONAL_CHARS {
        return Some(line);
    }
    // One more character says whether the cut falls between two words.
    let cut: String = line.chars().take(PROVISIONAL_CHARS + 1).collect();
    let words = cut
        .rsplit_once(' ')
        .map_or(cut.as_str(), |(head, _)| head)
        .trim_end_matches([',', '.', ';', ':']);
    Some(format!("{words}…"))
}

/// What the AI answered, as a title: one line, at most six words, without
/// quotes or a final stop.
fn clean(raw: &str) -> Option<String> {
    let line = raw.lines().map(str::trim).find(|line| !line.is_empty())?;
    let line = line.trim_matches(|c: char| matches!(c, '"' | '\'' | '“' | '”' | '«' | '»' | '*'));
    let words: Vec<&str> = line.split_whitespace().take(MAX_WORDS).collect();
    let title = words
        .join(" ")
        .trim_end_matches(['.', '!', '?', ':', ';', ','])
        .chars()
        .take(MAX_CHARS)
        .collect::<String>();
    (!title.is_empty()).then_some(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_is_a_few_words_on_one_line() {
        assert_eq!(
            clean("\"Planejando viagem curta a Lisboa.\"\n").as_deref(),
            Some("Planejando viagem curta a Lisboa")
        );
        assert_eq!(
            clean("One two three four five six seven eight").as_deref(),
            Some("One two three four five six")
        );
        assert_eq!(clean("\n\n  "), None);
    }

    #[test]
    fn meanwhile_the_conversation_is_called_by_the_start_of_the_message() {
        assert_eq!(start_of("Plan the trip").as_deref(), Some("Plan the trip"));
        assert_eq!(
            start_of("I want to plan a five day trip to Lisbon in March, cheaply").as_deref(),
            Some("I want to plan a five day trip to Lisbon…")
        );
        assert_eq!(start_of("   "), None);
    }
}
