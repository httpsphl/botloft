//! Each bot's own browser (spec 21): a Microsoft Edge without a window that
//! the daemon drives over the DevTools protocol. It starts with the bot's
//! first browser tool, closes when nobody uses or watches it, and sends
//! live frames to the app connection that watches it.

mod agent;
mod call;
mod cdp;
mod events;
mod hands;
mod input;
mod keys;
mod launch;
mod lesson;
mod moves;
mod owner_open;
mod page;
mod program;
mod read;
mod rest;
mod session;
mod settle;
pub mod sites;
mod sweep;
mod titles;
mod viewport;
mod watch;
mod window;

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use botloft_core::ids::{ApprovalId, BotId};
use botloft_core::protocol::{
    BrowserAction, BrowserControl, BrowserFrame, BrowserState, BrowserStatus, BrowserView,
};
use tokio::sync::broadcast;
use tracing::{debug, warn};

pub use self::call::Call;
pub use self::hands::{Asking, Hands, InputError, OWNER_WAIT, TakeError};
pub use self::keys::{Key, find as find_key, names as key_names};
pub use self::page::{Aim, Done, Scroll};
pub use self::program::find as find_program;
pub use self::read::{READ_MAX, Reading};
pub use self::session::Session;
pub use self::sweep::{Want, run, want};
pub use self::viewport::Viewport;
pub use self::watch::Watching;
pub use self::window::WindowError;
use crate::clock::Clock;
use crate::config::Config;
use crate::state::Event;

#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("{}", program::NOT_FOUND)]
    NotFound,
    #[error("the browser could not start: {0}")]
    Start(String),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Cdp(#[from] cdp::CdpError),
    #[error("the browser has no page open")]
    NoPage,
    #[error("{0} is not on the page anymore; call browser_look to read the page again")]
    Stale(String),
    #[error("the page did not open: {0}")]
    Navigation(String),
    #[error("{0}")]
    Page(String),
}

/// `[browser]` from the config (spec 21.9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserSettings {
    /// Empty means the Edge that comes with Windows.
    pub path: String,
    pub idle: Duration,
    pub max_open: usize,
}

impl BrowserSettings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            path: config.browser.path.clone(),
            idle: Duration::from_secs(config.browser.idle_minutes * 60),
            max_open: config.browser.max_open,
        }
    }
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self::from_config(&Config::default())
    }
}

type Frames = tokio::sync::watch::Sender<Option<Arc<BrowserFrame>>>;
/// Which hold of the owner's has the browser, if any (spec 21.10).
type Held = tokio::sync::watch::Sender<Option<u64>>;

struct Slot {
    /// One tool call at a time in each browser.
    calls: Arc<tokio::sync::Mutex<()>>,
    /// The running browser and which start it was.
    session: Option<(u64, Arc<Session>)>,
    state: BrowserState,
    frames: Frames,
    watchers: usize,
    /// The page size the app watching asked for (spec 21.3).
    viewport: Viewport,
    used: Instant,
    held: Held,
    /// The bot's open request for the owner's help.
    asking: Option<ApprovalId>,
    /// The owner has the profile open in a window of its own.
    window: Option<window::Window>,
    /// The page it showed last, to open again for the owner.
    last_url: Option<String>,
}

type Slots = Arc<Mutex<HashMap<BotId, Slot>>>;

fn lock(slots: &Slots) -> MutexGuard<'_, HashMap<BotId, Slot>> {
    slots
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub struct Browsers {
    settings: BrowserSettings,
    /// `<home>\browsers`: one profile folder per bot.
    profiles: PathBuf,
    events: broadcast::Sender<Event>,
    clock: Arc<dyn Clock>,
    slots: Slots,
    starts: std::sync::atomic::AtomicU64,
    holds: std::sync::atomic::AtomicU64,
}

impl Browsers {
    pub fn new(
        settings: BrowserSettings,
        home: &Path,
        events: broadcast::Sender<Event>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            settings,
            profiles: home.join("browsers"),
            events,
            clock,
            slots: Slots::default(),
            starts: std::sync::atomic::AtomicU64::new(1),
            holds: std::sync::atomic::AtomicU64::new(1),
        }
    }

    fn slot<'a>(&self, slots: &'a mut HashMap<BotId, Slot>, bot: &BotId) -> &'a mut Slot {
        slots.entry(bot.clone()).or_insert_with(|| Slot {
            calls: Arc::default(),
            session: None,
            state: BrowserState::closed(bot.clone(), self.clock.now_ms()),
            frames: tokio::sync::watch::channel(None).0,
            watchers: 0,
            viewport: Viewport::default(),
            used: Instant::now(),
            held: tokio::sync::watch::channel(None).0,
            asking: None,
            window: None,
            last_url: None,
        })
    }

    /// Waits for the bot's browser to be free and holds it for one call,
    /// awake.
    pub async fn begin(&self, bot: &BotId) -> Call<'_> {
        let calls = {
            let mut slots = lock(&self.slots);
            Arc::clone(&self.slot(&mut slots, bot).calls)
        };
        let guard = calls.lock_owned().await;
        self.touch(bot);
        self.wake(bot).await;
        Call {
            browsers: self,
            bot: bot.clone(),
            _guard: guard,
        }
    }

    pub(super) fn touch(&self, bot: &BotId) {
        if let Some(slot) = lock(&self.slots).get_mut(bot) {
            slot.used = Instant::now();
        }
    }

    /// The running browser of `bot`, if any.
    fn running(&self, bot: &BotId) -> Option<Arc<Session>> {
        lock(&self.slots)
            .get(bot)
            .and_then(|slot| slot.session.as_ref())
            .filter(|(_, session)| !session.is_closed())
            .map(|(_, session)| Arc::clone(session))
    }

    fn set_state(&self, bot: &BotId, change: impl FnOnce(&mut BrowserState)) {
        update(&self.slots, &self.events, self.clock.as_ref(), bot, change);
    }

    /// Closes the bot's browser; the profile stays. The owner's hands let
    /// go of it too, but not a window of its own they have open: the bot
    /// keeps waiting for that one (spec 21.11).
    pub fn close(&self, bot: &BotId) {
        let session = lock(&self.slots).get_mut(bot).and_then(|slot| {
            if slot.window.is_none() {
                slot.held.send_replace(None);
            }
            slot.session.take()
        });
        if let Some((_, session)) = session {
            debug!(bot = %bot, "browser: closing");
            session.close();
        }
        self.set_state(bot, |state| {
            let window = state.window;
            *state = BrowserState::closed(state.bot_id.clone(), state.updated_at);
            if window {
                state.window = true;
                state.control = BrowserControl::Owner;
            }
        });
    }

    /// Closes the browser, a window of it too, and deletes its profile:
    /// the bot was archived.
    pub fn forget(&self, bot: &BotId) {
        if let Some(window) = lock(&self.slots)
            .get_mut(bot)
            .and_then(|slot| slot.window.take())
        {
            window.kill();
        }
        self.close(bot);
        let profile = self.profiles.join(bot.as_str());
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            let _ = std::fs::remove_dir_all(&profile);
            return;
        };
        runtime.spawn(async move {
            // The processes may take a moment to let go of their files.
            for _ in 0..10 {
                match std::fs::remove_dir_all(&profile) {
                    Ok(()) => return,
                    Err(err) if err.kind() == io::ErrorKind::NotFound => return,
                    Err(_) => tokio::time::sleep(Duration::from_millis(500)).await,
                }
            }
            warn!("browser: could not delete an archived bot's browser profile");
        });
    }

    /// The browsers that are not closed, or open in a window of their own
    /// (`browser.list`).
    pub fn list(&self) -> Vec<BrowserState> {
        lock(&self.slots)
            .values()
            .map(|slot| slot.state.clone())
            .filter(|state| state.status != BrowserStatus::Closed || state.window)
            .collect()
    }

    pub fn view(&self, bot: &BotId) -> BrowserView {
        let mut slots = lock(&self.slots);
        let slot = self.slot(&mut slots, bot);
        BrowserView {
            state: slot.state.clone(),
            frame: slot.frames.borrow().as_deref().cloned(),
        }
    }

    /// Tells every app what the bot just did, for the cursor.
    pub fn action(&self, action: BrowserAction) {
        let _ = self.events.send(Event::BrowserAction(action));
    }
}

/// Changes a bot's browser state and tells the apps when it changed.
fn update(
    slots: &Slots,
    events: &broadcast::Sender<Event>,
    clock: &dyn Clock,
    bot: &BotId,
    change: impl FnOnce(&mut BrowserState),
) {
    let changed = {
        let mut all = lock(slots);
        let Some(slot) = all.get_mut(bot) else {
            return;
        };
        let before = slot.state.clone();
        change(&mut slot.state);
        (slot.state != before).then(|| {
            slot.state.updated_at = clock.now_ms();
            slot.state.clone()
        })
    };
    if let Some(state) = changed {
        let _ = events.send(Event::BrowserChanged(state));
    }
}
