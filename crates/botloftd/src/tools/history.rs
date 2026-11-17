//! `search_history` (spec 8.8): a bot finds words in its own chat, and
//! reads the talk around a hit, for what is no longer in its context.
//! Only its own chat: never another bot's talk with the owner.

use botloft_core::ids::{BotId, ChatItemId};
use botloft_core::protocol::{ChatItem, SNIPPET_CLOSE, SNIPPET_OPEN};
use botloft_store::{SearchFilter, Store, fts_query};
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::explain;
use super::history_text::{Teller, conversational, cut, kind_of, text_of};
use crate::state::Daemon;

pub const SEARCH_HISTORY: &str = "search_history";

const LIMIT_DEFAULT: u32 = 10;
const LIMIT_MAX: u32 = 20;
const QUERY_MAX_CHARS: usize = 200;
/// Fewest letters worth searching for, as in the owner's search.
const QUERY_MIN_CHARS: usize = 2;
const SNIPPET_WORDS: u32 = 40;
/// Items of the talk read on each side of `around`.
const AROUND_EACH_SIDE: usize = 6;
/// Items looked at on each side to find those, tools and notices included.
const AROUND_SCAN: u32 = 300;
const TEXT_MAX_CHARS: usize = 2_000;

pub(super) fn tool() -> Value {
    json!({
        "name": SEARCH_HISTORY,
        "title": "Search your past conversations",
        "description": "Finds words in your own chat, for what is no longer in your context: \
            what the owner, other bots and Botloft wrote to you, what you replied, and your \
            questions and notes with their answers. Use it when someone refers to something from \
            before (a decision, a name, a number, a file) that you do not have, before you ask \
            again or guess. Every word must be there, in any order; accents and case do not \
            matter, and the last word may be the start of a word. Hits come newest first, with \
            an excerpt and the conversation they are in: `main`, or the title of another one \
            the owner opened with you. Then call it with `around` set to a hit's item_id to \
            read the talk there, in that conversation. It never sees other bots' chats.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "maxLength": QUERY_MAX_CHARS,
                    "description": "A few words to find, such as a name or a topic.",
                },
                "before": {
                    "type": "string",
                    "description": "The item_id of the last hit you got, for older hits.",
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": LIMIT_MAX,
                    "description": "How many hits; 10 if left out.",
                },
                "around": {
                    "type": "string",
                    "description": "Instead of query: a hit's item_id, to read the messages and \
                        replies just before and after it.",
                },
            },
            "additionalProperties": false,
        },
        "annotations": { "readOnlyHint": true },
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Args {
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    before: Option<String>,
    #[serde(default)]
    limit: Option<u32>,
    #[serde(default)]
    around: Option<String>,
}

pub(super) fn call(daemon: &Daemon, bot: &BotId, args: Args) -> Result<Value, String> {
    match (args.query, args.around) {
        (Some(query), None) => search(daemon, bot, &query, args.before.as_deref(), args.limit),
        (None, Some(around)) if args.before.is_none() && args.limit.is_none() => {
            read_around(daemon, bot, &around)
        }
        (None, Some(_)) => Err("before and limit go with query, not with around".to_owned()),
        _ => Err(
            "pass either query, to find words, or around, with an item_id a search gave you"
                .to_owned(),
        ),
    }
}

fn search(
    daemon: &Daemon,
    bot: &BotId,
    query: &str,
    before: Option<&str>,
    limit: Option<u32>,
) -> Result<Value, String> {
    let limit = limit.unwrap_or(LIMIT_DEFAULT);
    if !(1..=LIMIT_MAX).contains(&limit) {
        return Err(format!("limit must be between 1 and {LIMIT_MAX}"));
    }
    if query.chars().count() > QUERY_MAX_CHARS {
        return Err(format!("query takes at most {QUERY_MAX_CHARS} characters"));
    }
    let letters = query.chars().filter(|c| c.is_alphanumeric()).count();
    let words = fts_query(query)
        .filter(|_| letters >= QUERY_MIN_CHARS)
        .ok_or_else(|| format!("query needs at least {QUERY_MIN_CHARS} letters or digits"))?;
    let before = before.map(|id| item_id(id, "before")).transpose()?;

    let store = daemon.store();
    let filter = SearchFilter {
        bot: Some(bot),
        crew: None,
        before: before.as_ref(),
        limit,
        snippet_words: Some(SNIPPET_WORDS),
    };
    let found = store
        .search_chat(&words, filter)
        .map_err(|err| explain(err.into()))?;
    let teller = Teller::new(&store);
    let hits: Vec<Value> = found
        .iter()
        .map(|hit| {
            json!({
                "item_id": hit.item.id,
                "at": teller.at(hit.item.created_at),
                "from": teller.from(&hit.item),
                "kind": kind_of(&hit.item),
                "conversation": conversation_of(&store, &hit.item),
                "excerpt": marked(&hit.snippet),
            })
        })
        .collect();
    let more = hits.len() == limit as usize;
    let note = if hits.is_empty() {
        "Nothing in your chat has all these words. Try fewer or other words; the owner may \
            also have cleared older talk."
            .to_owned()
    } else {
        format!(
            "Times are in {}. Call search_history with around set to an item_id to read the \
                talk there{}.",
            teller.zone(),
            if more {
                "; with before set to the last item_id for older hits"
            } else {
                ""
            }
        )
    };
    Ok(json!({ "query": query, "hits": hits, "more": more, "note": note }))
}

fn read_around(daemon: &Daemon, bot: &BotId, around: &str) -> Result<Value, String> {
    let id = item_id(around, "around")?;
    let store = daemon.store();
    let not_here = || format!("{around} is not in your chat; use an item_id a search gave you");
    let item = store
        .chat_item(&id)
        .map_err(|err| explain(err.into()))?
        .filter(|item| &item.bot_id == bot)
        .ok_or_else(not_here)?;
    // The talk around it is in its own conversation (spec 40.5).
    let older = store
        .chat_history_in(&item.conversation_id, Some(&id), AROUND_SCAN)
        .map_err(|err| explain(err.into()))?;
    let newer = store
        .chat_after_in(&item.conversation_id, &id, AROUND_SCAN)
        .map_err(|err| explain(err.into()))?;

    let mut talk: Vec<&ChatItem> = older
        .iter()
        .filter(|item| conversational(item))
        .take(AROUND_EACH_SIDE)
        .collect();
    talk.reverse();
    talk.push(&item);
    talk.extend(
        newer
            .iter()
            .filter(|item| conversational(item))
            .take(AROUND_EACH_SIDE),
    );

    let teller = Teller::new(&store);
    let items: Vec<Value> = talk
        .iter()
        .map(|each| {
            let mut told = json!({
                "item_id": each.id,
                "at": teller.at(each.created_at),
                "from": teller.from(each),
                "kind": kind_of(each),
                "text": cut(&text_of(each), TEXT_MAX_CHARS),
            });
            if each.id == id {
                told["this"] = json!(true);
            }
            told
        })
        .collect();
    Ok(json!({
        "around": id,
        "conversation": conversation_of(&store, &item),
        "items": items,
        "note": format!(
            "Oldest first; the item you asked for has \"this\": true. Times are in {}. Tool calls \
                and notices are left out.",
            teller.zone()
        ),
    }))
}

/// Which of the bot's conversations an item is in (spec 40): `main`, or
/// the other one's title.
fn conversation_of(store: &Store, item: &ChatItem) -> String {
    if item.conversation_id.is_main_of(&item.bot_id) {
        return "main".to_owned();
    }
    let title = store
        .conversation(&item.conversation_id)
        .ok()
        .flatten()
        .and_then(|conversation| conversation.title);
    title.unwrap_or_else(|| "another conversation".to_owned())
}

fn item_id(text: &str, field: &str) -> Result<ChatItemId, String> {
    text.trim()
        .parse()
        .map_err(|_| format!("{field} must be an item_id that search_history gave you"))
}

/// The snippet with each found word in bold, as markdown.
fn marked(snippet: &str) -> String {
    snippet.replace([SNIPPET_OPEN, SNIPPET_CLOSE], "**")
}
