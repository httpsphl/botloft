//! The design area (spec 22): the key the app loads a bot's screens with,
//! and the drafts of the HTML files bots are writing right now, read from
//! the tool input Claude Code streams while the model writes it.

mod partial;
mod place;
mod view;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use botloft_core::ids::BotId;
use botloft_core::protocol::ScreenDraft;
use serde_json::Value;

pub use self::place::{Place, Root, device_of, locate, roots};
pub use self::view::handle as view;
use crate::state::{Daemon, Event};

/// The most often a draft goes out while it is being written.
const DRAFT_EVERY: Duration = Duration::from_millis(250);

#[derive(Default)]
pub struct Screens {
    keys: Mutex<HashMap<BotId, String>>,
    drafts: Mutex<HashMap<BotId, BotDrafts>>,
}

#[derive(Default)]
struct BotDrafts {
    /// `Write` blocks being written, by their index in the message.
    writing: HashMap<u64, Stream>,
    /// Drafts by where they are served.
    by_place: HashMap<Place, Draft>,
}

struct Stream {
    tool_use_id: String,
    json: String,
    sent: Option<Instant>,
}

struct Draft {
    tool_use_id: String,
    path: String,
    content: String,
    rev: u64,
}

impl Screens {
    fn drafts(&self) -> MutexGuard<'_, HashMap<BotId, BotDrafts>> {
        self.drafts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_keys(&self) -> MutexGuard<'_, HashMap<BotId, String>> {
        self.keys
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The bot's key for `/view`, made on first use (spec 22.2).
    pub fn key(&self, bot: &BotId) -> String {
        self.lock_keys()
            .entry(bot.clone())
            .or_insert_with(|| {
                let mut bytes = [0u8; 16];
                // Without randomness no key could be trusted; an empty one
                // matches no request.
                match getrandom::fill(&mut bytes) {
                    Ok(()) => hex::encode(bytes),
                    Err(_) => String::new(),
                }
            })
            .clone()
    }

    /// The bot a key belongs to.
    pub fn bot_of(&self, key: &str) -> Option<BotId> {
        if key.is_empty() {
            return None;
        }
        self.lock_keys()
            .iter()
            .find(|(_, known)| known.as_str() == key)
            .map(|(bot, _)| bot.clone())
    }

    /// What is served at `place` while the bot writes it.
    pub fn draft(&self, bot: &BotId, place: &Place) -> Option<String> {
        self.drafts()
            .get(bot)?
            .by_place
            .get(place)
            .map(|draft| draft.content.clone())
    }

    /// The drafts of `bot`: where, the file and its content.
    pub fn drafts_of(&self, bot: &BotId) -> Vec<(Place, String, String)> {
        self.drafts().get(bot).map_or_else(Vec::new, |drafts| {
            drafts
                .by_place
                .iter()
                .map(|(place, draft)| (place.clone(), draft.path.clone(), draft.content.clone()))
                .collect()
        })
    }
}

/// Follows the `Write` blocks of a `stream_event` (spec 22.3).
pub fn stream(daemon: &Daemon, bot: &BotId, event: &Value) {
    let inner = &event["event"];
    let index = inner["index"].as_u64().unwrap_or_default();
    match inner["type"].as_str() {
        Some("content_block_start") => {
            let block = &inner["content_block"];
            if block["type"] == "tool_use"
                && block["name"] == "Write"
                && let Some(id) = block["id"].as_str()
            {
                let stream = Stream {
                    tool_use_id: id.to_owned(),
                    json: String::new(),
                    sent: None,
                };
                let mut drafts = daemon.screens.drafts();
                drafts
                    .entry(bot.clone())
                    .or_default()
                    .writing
                    .insert(index, stream);
            }
        }
        Some("content_block_delta") if inner["delta"]["type"] == "input_json_delta" => {
            let due = {
                let mut drafts = daemon.screens.drafts();
                let Some(stream) = drafts
                    .get_mut(bot)
                    .and_then(|drafts| drafts.writing.get_mut(&index))
                else {
                    return;
                };
                stream
                    .json
                    .push_str(inner["delta"]["partial_json"].as_str().unwrap_or_default());
                stream.sent.is_none_or(|sent| sent.elapsed() >= DRAFT_EVERY)
            };
            if due {
                update(daemon, bot, index, false);
            }
        }
        Some("content_block_stop") => update(daemon, bot, index, true),
        _ => {}
    }
}

/// Turns what arrived of a `Write` into a draft and tells the apps.
fn update(daemon: &Daemon, bot: &BotId, index: u64, stopped: bool) {
    let (tool_use_id, json) = {
        let mut drafts = daemon.screens.drafts();
        let Some(writing) = drafts.get_mut(bot).map(|drafts| &mut drafts.writing) else {
            return;
        };
        let stream = if stopped {
            writing.remove(&index)
        } else {
            writing.get(&index).map(|stream| Stream {
                tool_use_id: stream.tool_use_id.clone(),
                json: stream.json.clone(),
                sent: None,
            })
        };
        let Some(stream) = stream else {
            return;
        };
        (stream.tool_use_id, stream.json)
    };
    let fields = partial::fields(&json);
    let Some(path) = fields
        .get("file_path")
        .filter(|path| path.complete)
        .map(|path| path.value.clone())
    else {
        return;
    };
    let Some(place) = place::locate(daemon, bot, &PathBuf::from(&path)) else {
        return;
    };
    let content = fields
        .get("content")
        .map(|content| content.value.clone())
        .unwrap_or_default();
    let rev = {
        let mut drafts = daemon.screens.drafts();
        let by_place = &mut drafts.entry(bot.clone()).or_default().by_place;
        let draft = by_place.entry(place.clone()).or_insert_with(|| Draft {
            tool_use_id: tool_use_id.clone(),
            path: path.clone(),
            content: String::new(),
            rev: 0,
        });
        draft.tool_use_id = tool_use_id;
        draft.content = content;
        draft.rev += 1;
        draft.rev
    };
    tell(daemon, bot, &place, path, rev, false);
    // Until the path arrives nothing goes out, so the wait starts here.
    if let Some(stream) = daemon
        .screens
        .drafts()
        .get_mut(bot)
        .and_then(|drafts| drafts.writing.get_mut(&index))
    {
        stream.sent = Some(Instant::now());
    }
}

/// A tool finished: its draft, if it had one, gives way to the file.
pub fn tool_done(daemon: &Daemon, bot: &BotId, tool_use_id: &str) {
    let done: Vec<(Place, Draft)> = {
        let mut drafts = daemon.screens.drafts();
        let Some(drafts) = drafts.get_mut(bot) else {
            return;
        };
        let places: Vec<Place> = drafts
            .by_place
            .iter()
            .filter(|(_, draft)| draft.tool_use_id == tool_use_id)
            .map(|(place, _)| place.clone())
            .collect();
        places
            .into_iter()
            .filter_map(|place| drafts.by_place.remove(&place).map(|draft| (place, draft)))
            .collect()
    };
    for (place, draft) in done {
        tell(daemon, bot, &place, draft.path, draft.rev + 1, true);
    }
}

/// The turn ended or the process did: nothing more will be written.
pub fn turn_ended(daemon: &Daemon, bot: &BotId) {
    let left = daemon.screens.drafts().remove(bot);
    for (place, draft) in left.map(|drafts| drafts.by_place).unwrap_or_default() {
        tell(daemon, bot, &place, draft.path, draft.rev + 1, true);
    }
}

fn tell(daemon: &Daemon, bot: &BotId, place: &Place, path: String, rev: u64, done: bool) {
    let key = daemon.screens.key(bot);
    let url = place.url(daemon.port, &key, &format!("rev={rev}"));
    daemon.emit(Event::ScreenDraft(ScreenDraft {
        bot_id: bot.clone(),
        path,
        url,
        rev,
        done,
    }));
}
