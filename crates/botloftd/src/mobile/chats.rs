//! What the phone is shown of the conversations (spec 28.12): the list of
//! bots, and the items of a chat as text only, cut and split to fit the
//! sealed message.

use botloft_core::ids::{BotId, ChatItemId};
use botloft_core::protocol::{
    ChatBody, ChatItem, ChatLine, ITEM_TEXT_MAX, PhoneItem, SEALED_MAX, SenderKind, ToPhone,
};
use botloft_store::Store;
use serde::Serialize;

use crate::service::bots;
use crate::state::Daemon;

/// Items in a page of history.
pub const PAGE: u32 = 20;

/// What one message of a long answer may hold, leaving room for the frame
/// around it.
const PART_ROOM: usize = SEALED_MAX - 1024;

/// `text` cut to the phone's limit on a character boundary, and whether it
/// was cut.
pub fn cut(text: &str) -> (String, bool) {
    if text.len() <= ITEM_TEXT_MAX {
        return (text.to_owned(), false);
    }
    let mut end = ITEM_TEXT_MAX;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (text[..end].to_owned(), true)
}

/// The last `ITEM_TEXT_MAX` bytes of `text`: a reply being written is read
/// at its end.
pub fn tail(text: &str) -> String {
    if text.len() <= ITEM_TEXT_MAX {
        return text.to_owned();
    }
    let mut start = text.len() - ITEM_TEXT_MAX;
    while !text.is_char_boundary(start) {
        start += 1;
    }
    text[start..].to_owned()
}

/// The line of one active bot; `None` for one that is archived or gone.
pub fn line(daemon: &Daemon, bot: &BotId) -> Option<ChatLine> {
    let store = daemon.store();
    let (crew, record) = bots::active(&store, bot).ok()?;
    let crew_name = crew.name.clone();
    let bot = bots::to_protocol(daemon, &store, &crew, record);
    Some(ChatLine {
        bot_id: bot.id,
        name: bot.name,
        color: bot.color,
        crew: crew_name,
        state: bot.state,
        last_reply_at: bot.last_reply_at,
        last: bot.last_activity,
    })
}

/// Every active bot, the one with the newest activity first.
pub fn lines(daemon: &Daemon) -> Vec<ChatLine> {
    let ids: Vec<BotId> = daemon
        .store()
        .bots(None, false)
        .unwrap_or_default()
        .into_iter()
        .map(|record| record.id)
        .collect();
    let mut lines: Vec<ChatLine> = ids.iter().filter_map(|id| line(daemon, id)).collect();
    lines.sort_by_key(|line| std::cmp::Reverse(line.last.as_ref().map_or(0, |last| last.at)));
    lines
}

/// Who wrote a message to a bot: the bot, the routine or the daemon.
fn sender(store: &Store, item: &ChatItem) -> String {
    let ChatBody::Inbound(inbound) = &item.body else {
        return String::new();
    };
    let message = &inbound.message;
    let routine = message
        .routine_id
        .as_ref()
        .and_then(|id| store.routine(id).ok().flatten())
        .map(|routine| routine.name);
    let bot = message
        .from_bot_id
        .as_ref()
        .and_then(|id| store.bot(id).ok().flatten())
        .map(|bot| bot.name);
    routine.or(bot).unwrap_or_else(|| "Botloft".to_owned())
}

/// The item as the phone shows it; `None` for what it does not show (a
/// turn that worked).
pub fn item(daemon: &Daemon, item: &ChatItem) -> Option<PhoneItem> {
    let (id, at) = (item.id.clone(), item.created_at);
    Some(match &item.body {
        ChatBody::Inbound(inbound) => {
            let (text, cut) = cut(&inbound.message.body);
            if inbound.message.from_kind == SenderKind::Owner {
                PhoneItem::You { id, at, text, cut }
            } else {
                let from = sender(&daemon.store(), item);
                PhoneItem::BotMessage {
                    id,
                    at,
                    from,
                    text,
                    cut,
                }
            }
        }
        ChatBody::Reply(reply) => {
            let (text, cut) = cut(&reply.text);
            PhoneItem::Reply { id, at, text, cut }
        }
        ChatBody::Tool(tool) => PhoneItem::Tool {
            id,
            at,
            summary: cut(&tool.summary).0,
        },
        ChatBody::Approval(shown) => PhoneItem::Approval {
            id,
            at,
            approval_id: shown.approval_id.clone(),
            summary: cut(&shown.summary).0,
            status: shown.status,
        },
        ChatBody::Question(shown) => PhoneItem::Question {
            id,
            at,
            question_id: shown.question.id.clone(),
            text: cut(&shown.question.text).0,
            status: shown.question.status,
        },
        ChatBody::Turn(turn) => {
            let error = turn.error.as_deref()?;
            PhoneItem::Failed {
                id,
                at,
                error: Some(cut(error).0),
            }
        }
        ChatBody::Notice(notice) => PhoneItem::Notice {
            id,
            at,
            level: notice.level,
            text: cut(&notice.text).0,
        },
    })
}

/// A page of the chat, oldest first, and whether there are older ones.
pub fn page(
    daemon: &Daemon,
    bot: &BotId,
    before: Option<&ChatItemId>,
) -> Option<(Vec<PhoneItem>, bool)> {
    bots::active(&daemon.store(), bot).ok()?;
    let mut found = daemon
        .store()
        .chat_history(bot, before, PAGE + 1)
        .unwrap_or_default();
    let more = found.len() > PAGE as usize;
    found.truncate(PAGE as usize);
    found.reverse();
    let items = found
        .iter()
        .filter_map(|found| item(daemon, found))
        .collect();
    Some((items, more))
}

/// Splits `things` into groups that each fit one sealed message.
fn groups<T: Serialize>(things: Vec<T>) -> Vec<Vec<T>> {
    let mut groups = vec![Vec::new()];
    let mut room = PART_ROOM;
    for thing in things {
        let size = serde_json::to_vec(&thing).map_or(PART_ROOM, |bytes| bytes.len() + 1);
        let group = groups.last().is_some_and(|group| !group.is_empty());
        if group && size > room {
            groups.push(Vec::new());
            room = PART_ROOM;
        }
        room = room.saturating_sub(size);
        if let Some(last) = groups.last_mut() {
            last.push(thing);
        }
    }
    groups
}

/// The list as the messages that carry it.
pub fn chats_parts(lines: Vec<ChatLine>) -> Vec<ToPhone> {
    groups(lines)
        .into_iter()
        .enumerate()
        .map(|(index, bots)| ToPhone::Chats {
            bots,
            first: index == 0,
        })
        .collect()
}

/// A page of history as the messages that carry it.
pub fn history_parts(req: u32, bot: &BotId, items: Vec<PhoneItem>, more: bool) -> Vec<ToPhone> {
    let all = groups(items);
    let last = all.len() - 1;
    all.into_iter()
        .enumerate()
        .map(|(index, items)| ToPhone::History {
            req,
            bot_id: bot.clone(),
            items,
            more,
            done: index == last,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_text_is_cut_on_a_character_and_marked() {
        assert_eq!(cut("short"), ("short".to_owned(), false));
        let (text, was_cut) = cut(&"é".repeat(ITEM_TEXT_MAX));
        assert!(was_cut && text.len() <= ITEM_TEXT_MAX && text.chars().all(|c| c == 'é'));
        let end = tail(&format!("{}fim", "é".repeat(ITEM_TEXT_MAX)));
        assert!(end.len() <= ITEM_TEXT_MAX && end.ends_with("fim"));
    }

    #[test]
    fn a_big_page_goes_in_parts_that_each_fit() {
        let items: Vec<PhoneItem> = (0..20)
            .map(|n| PhoneItem::Reply {
                id: ChatItemId::generate(),
                at: n,
                text: "x".repeat(ITEM_TEXT_MAX),
                cut: false,
            })
            .collect();
        let bot = BotId::generate();
        let parts = history_parts(7, &bot, items, true);
        assert!(parts.len() > 1);
        let mut count = 0;
        for (index, part) in parts.iter().enumerate() {
            let size = serde_json::to_vec(part).expect("json").len();
            assert!(size <= SEALED_MAX, "part {index} is {size} bytes");
            let ToPhone::History { items, done, .. } = part else {
                panic!("not history");
            };
            count += items.len();
            assert_eq!(*done, index == parts.len() - 1);
        }
        assert_eq!(count, 20);
        // An empty page is still one message, and it is the last.
        let none = history_parts(1, &bot, Vec::new(), false);
        assert!(matches!(
            none.as_slice(),
            [ToPhone::History { done: true, .. }]
        ));
    }
}
