//! The history of each bot's memory and instructions (spec 8.10). The
//! memory is the `CLAUDE.md` in the bot's folder, which the bot edits: the
//! daemon looks at it when a turn begins and when it ends. What changed
//! during the turn is the bot's; what changed between turns came from
//! outside Botloft. The instructions are saved where they change
//! (`service::bots::update_by`). The history stays in the daemon's
//! database, out of the bot's folder, so the bot cannot rewrite it.

use std::fs;
use std::io;
use std::path::PathBuf;

use botloft_core::ids::{BotId, VersionId};
use botloft_core::protocol::{BotState, TextVersion, VersionAuthor, VersionKind};
use tracing::{debug, warn};

use crate::service::bots;
use crate::state::{Daemon, Event};

/// The bot's memory, in its folder (spec 5.1).
pub const MEMORY_FILE: &str = "CLAUDE.md";
/// The biggest memory file kept whole in a version; a bigger one is left
/// out of the history.
const MEMORY_MAX_BYTES: u64 = 256 * 1024;

/// Who made a change, and the version it went back to when it was an Undo.
#[derive(Debug, Clone)]
pub struct Origin {
    pub author: VersionAuthor,
    pub restored_from: Option<VersionId>,
}

impl Origin {
    pub fn by(author: VersionAuthor) -> Self {
        Self {
            author,
            restored_from: None,
        }
    }
}

/// A turn begins: a change since the last look was made outside Botloft.
pub fn turn_began(daemon: &Daemon, bot: &BotId) {
    look(daemon, bot, VersionAuthor::Outside);
}

/// A turn ended: a change since it began is the bot's.
pub fn turn_ended(daemon: &Daemon, bot: &BotId) {
    look(daemon, bot, VersionAuthor::Agent);
}

/// The owner looks at the history: a change since the last turn came from
/// outside. While the bot is in a turn, its own change waits for the turn's
/// end, so it is not taken for someone else's.
pub fn look_between_turns(daemon: &Daemon, bot: &BotId) {
    let in_turn = matches!(
        daemon.supervisor.status(bot),
        Some((BotState::Busy | BotState::NeedsApproval, _))
    );
    if !in_turn {
        look(daemon, bot, VersionAuthor::Outside);
    }
}

/// The bot's memory file, while the bot and its crew are active.
pub fn memory_path(daemon: &Daemon, bot: &BotId) -> Option<PathBuf> {
    let store = daemon.store();
    let (crew, record) = bots::active(&store, bot).ok()?;
    Some(
        daemon
            .paths
            .bot_workspace(&crew.slug, &record.slug)
            .join(MEMORY_FILE),
    )
}

/// The memory as it is now: empty when the file is not there, `None` when
/// it is too big to keep or cannot be read.
fn read_memory(path: &PathBuf) -> Option<String> {
    match fs::metadata(path) {
        Ok(meta) if meta.len() > MEMORY_MAX_BYTES => None,
        Ok(_) => fs::read_to_string(path).ok(),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Some(String::new()),
        Err(_) => None,
    }
}

fn look(daemon: &Daemon, bot: &BotId, author: VersionAuthor) {
    let Some(path) = memory_path(daemon, bot) else {
        return;
    };
    let Some(text) = read_memory(&path) else {
        debug!(bot = %bot, "the memory file is too big or unreadable to keep a version of");
        return;
    };
    record(daemon, bot, VersionKind::Memory, text, Origin::by(author));
}

/// Saves `text` as the bot's newest version of `kind`, unless it is what
/// the newest already says. The first time something is seen from outside,
/// it starts the history instead; an empty start is not kept.
pub fn record(
    daemon: &Daemon,
    bot: &BotId,
    kind: VersionKind,
    text: String,
    origin: Origin,
) -> Option<TextVersion> {
    let store = daemon.store();
    let latest = match store.latest_version(bot, kind) {
        Ok(latest) => latest,
        Err(err) => {
            warn!(bot = %bot, "could not read the history of a bot: {err}");
            return None;
        }
    };
    if latest.as_ref().is_some_and(|newest| newest.text == text) {
        return None;
    }
    let mut author = origin.author;
    if latest.is_none() && author == VersionAuthor::Outside {
        if text.is_empty() {
            return None;
        }
        author = VersionAuthor::Start;
    }
    let version = TextVersion {
        id: VersionId::generate(),
        bot_id: bot.clone(),
        kind,
        author,
        text,
        restored_from: origin.restored_from,
        created_at: daemon.clock.now_ms(),
    };
    if let Err(err) = store.insert_version(&version) {
        warn!(bot = %bot, "could not save a version: {err}");
        return None;
    }
    drop(store);
    debug!(bot = %bot, version = %version.id, kind = %kind, author = %author, "a new version");
    daemon.emit(Event::VersionCreated(version.clone()));
    Some(version)
}

/// The instructions changed from `before` to `after`: the first change keeps
/// how they were as the start of the history.
pub fn instructions_changed(
    daemon: &Daemon,
    bot: &BotId,
    before: &str,
    after: &str,
    origin: Origin,
) {
    if before == after {
        return;
    }
    let known = daemon
        .store()
        .latest_version(bot, VersionKind::Instructions)
        .ok()
        .flatten();
    if known.is_none() && !before.is_empty() {
        record(
            daemon,
            bot,
            VersionKind::Instructions,
            before.to_owned(),
            Origin::by(VersionAuthor::Start),
        );
    }
    record(
        daemon,
        bot,
        VersionKind::Instructions,
        after.to_owned(),
        origin,
    );
}
