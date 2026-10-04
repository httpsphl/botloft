//! The owner's desktop as the bots use it (spec 24): one action at a time
//! across every bot, whether the owner is there to see it, each bot's last
//! reading, whose refs its actions name, and what each bot does there, for
//! its panel, live.

mod activity;
pub mod notice;
mod screen;

pub use activity::{Activity, STOPPED};
pub use screen::DesktopWatching;

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use botloft_core::ids::BotId;
use botloft_core::protocol::DesktopView;
use tokio::sync::MutexGuard;

use self::screen::Screens;
use crate::platform::desktop::Control;

/// How long the owner may go without touching the computer and still count
/// as there (spec 24.8).
pub const AWAY_AFTER: Duration = Duration::from_secs(5 * 60);
/// How long the owner must leave the mouse and keyboard alone before a
/// bot takes them (spec 24.7), and after the owner just took them back.
pub const QUIET: Duration = Duration::from_secs(2);
pub const QUIET_AFTER_TAKEOVER: Duration = Duration::from_secs(10);
/// How long a takeover counts.
const TAKEOVER_LASTS: Duration = Duration::from_secs(10 * 60);

/// How long ago the owner last used the mouse or keyboard; `None` where
/// that cannot be known.
pub type OwnerIdle = Arc<dyn Fn() -> Option<Duration> + Send + Sync>;

/// A bot's last reading of a window: its refs are indexes in `controls`.
#[derive(Debug, Clone)]
pub struct Reading {
    pub window: u64,
    pub controls: Vec<Control>,
}

pub struct Desktop {
    /// App connections that said hello, open now.
    apps: AtomicUsize,
    readings: Mutex<HashMap<BotId, Reading>>,
    /// Held for each desktop action: the cursor is one.
    turn: tokio::sync::Mutex<()>,
    idle: Mutex<OwnerIdle>,
    pub activity: Arc<Activity>,
    screens: Arc<Screens>,
    /// When the owner last took the mouse or keyboard back from a bot.
    took_over: Mutex<Option<Instant>>,
    /// Each bot's last picture: the window and its size, for clicks on it.
    pictures: Mutex<HashMap<BotId, (u64, u32, u32)>>,
    /// The language the owner reads the app in, for the notice on screen.
    locale: Mutex<String>,
    /// Counts each real action, so the notice goes only after the last.
    notices: Arc<std::sync::atomic::AtomicU64>,
}

/// Why the bot may not use the desktop now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Away {
    /// Botloft's app is closed.
    NoApp,
    /// Nobody touched the computer for a while.
    Idle,
}

impl Away {
    /// What the bot reads.
    pub fn why(self) -> &'static str {
        match self {
            Self::NoApp => {
                "The owner does not have Botloft open, so you may not use their desktop now. \
                 Only grants they marked for use while they are away work then."
            }
            Self::Idle => {
                "The owner has not used the computer for a while, so you may not use their \
                 desktop now. Only grants they marked for use while they are away work then."
            }
        }
    }
}

impl Desktop {
    pub fn new(idle: OwnerIdle) -> Self {
        Self {
            apps: AtomicUsize::new(0),
            readings: Mutex::new(HashMap::new()),
            turn: tokio::sync::Mutex::new(()),
            idle: Mutex::new(idle),
            activity: Arc::default(),
            screens: Arc::default(),
            took_over: Mutex::new(None),
            pictures: Mutex::new(HashMap::new()),
            locale: Mutex::new("en".to_owned()),
            notices: Arc::default(),
        }
    }

    pub fn app_opened(&self) {
        self.apps.fetch_add(1, Ordering::SeqCst);
    }

    /// Always after its `app_opened`.
    pub fn app_closed(&self) {
        self.apps.fetch_sub(1, Ordering::SeqCst);
    }

    /// Changes how the owner's idle time is read: tests stand in for them.
    pub fn set_owner_idle(&self, idle: OwnerIdle) {
        *self.idle.lock().unwrap_or_else(PoisonError::into_inner) = idle;
    }

    /// Whether the owner is there to see what a bot does (spec 24.8): the
    /// app is open and they used the computer lately.
    pub fn owner_here(&self) -> Result<(), Away> {
        if self.apps.load(Ordering::SeqCst) == 0 {
            return Err(Away::NoApp);
        }
        let idle = self
            .idle
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        match idle() {
            Some(idle) if idle <= AWAY_AFTER => Ok(()),
            _ => Err(Away::Idle),
        }
    }

    /// Keeps `controls` as the bot's last reading of `window`.
    pub fn keep(&self, bot: &BotId, window: u64, controls: Vec<Control>) {
        self.readings
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(bot.clone(), Reading { window, controls });
    }

    /// The bot's last reading, if it read a window.
    pub fn reading(&self, bot: &BotId) -> Option<Reading> {
        self.readings
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(bot)
            .cloned()
    }

    /// Whether the owner leaves the mouse and keyboard alone, so a bot may
    /// take them (spec 24.7): a moment's quiet, longer right after the
    /// owner took them back.
    pub fn hands_free(&self) -> Result<(), String> {
        let took_over = *self
            .took_over
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let quiet = match took_over {
            Some(at) if at.elapsed() < TAKEOVER_LASTS => QUIET_AFTER_TAKEOVER,
            _ => QUIET,
        };
        let idle = self
            .idle
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        match idle() {
            Some(idle) if idle >= quiet => Ok(()),
            _ => Err(format!(
                "The owner is using the mouse or keyboard right now, so you may not take them. \
                 Wait until they have left them alone for {} seconds, then try again.",
                quiet.as_secs()
            )),
        }
    }

    /// The owner took the mouse or keyboard back in the middle of an action.
    pub fn owner_took_over(&self) {
        *self
            .took_over
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
    }

    /// Keeps the size of the bot's last picture of `window`.
    pub fn pictured(&self, bot: &BotId, window: u64, width: u32, height: u32) {
        self.pictures
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(bot.clone(), (window, width, height));
    }

    /// The size of the bot's last picture, if it was of `window`.
    pub fn picture_size(&self, bot: &BotId, window: u64) -> Option<(u32, u32)> {
        self.pictures
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(bot)
            .filter(|(id, _, _)| *id == window)
            .map(|&(_, width, height)| (width, height))
    }

    /// The language the owner reads the app in.
    pub fn set_locale(&self, locale: &str) {
        *self.locale.lock().unwrap_or_else(PoisonError::into_inner) = locale.to_owned();
    }

    pub fn locale(&self) -> String {
        self.locale
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// A new real action began: the number the notice's end waits on.
    pub fn notice_began(&self) -> u64 {
        self.notices.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// The count of real actions, for a wait that outlives the call.
    pub fn notices(&self) -> Arc<std::sync::atomic::AtomicU64> {
        Arc::clone(&self.notices)
    }

    /// Starts watching the window the bot is using (spec 24.9).
    pub fn watch(&self, bot: &BotId) -> DesktopWatching {
        self.screens.watch(bot, &self.activity)
    }

    /// The bot's state and the newest picture, for a panel that opens.
    pub fn view(&self, bot: &BotId) -> DesktopView {
        DesktopView {
            state: self.activity.state(bot),
            frame: self.screens.now(bot),
        }
    }

    /// Waits for the desktop to be free; it is the caller's until dropped.
    pub async fn turn(&self) -> MutexGuard<'_, ()> {
        self.turn.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idle_for(seconds: u64) -> OwnerIdle {
        Arc::new(move || Some(Duration::from_secs(seconds)))
    }

    #[test]
    fn the_owner_is_there_with_the_app_open_and_the_computer_in_use() {
        let desktop = Desktop::new(idle_for(10));
        assert_eq!(desktop.owner_here(), Err(Away::NoApp));
        desktop.app_opened();
        assert_eq!(desktop.owner_here(), Ok(()));
        desktop.set_owner_idle(idle_for(AWAY_AFTER.as_secs() + 1));
        assert_eq!(desktop.owner_here(), Err(Away::Idle));
        desktop.set_owner_idle(Arc::new(|| None));
        assert_eq!(desktop.owner_here(), Err(Away::Idle));
        desktop.set_owner_idle(idle_for(0));
        desktop.app_closed();
        assert_eq!(desktop.owner_here(), Err(Away::NoApp));
    }

    #[test]
    fn the_hands_are_free_after_a_moments_quiet_and_longer_after_a_takeover() {
        let desktop = Desktop::new(idle_for(1));
        assert!(desktop.hands_free().is_err(), "the owner is typing");
        desktop.set_owner_idle(idle_for(3));
        assert!(desktop.hands_free().is_ok());
        desktop.owner_took_over();
        let busy = desktop.hands_free().expect_err("just took over");
        assert!(busy.contains("10 seconds"), "{busy}");
        desktop.set_owner_idle(idle_for(11));
        assert!(desktop.hands_free().is_ok());
    }

    #[test]
    fn a_picture_size_is_for_its_own_window() {
        let desktop = Desktop::new(idle_for(0));
        let bot = BotId::generate();
        desktop.pictured(&bot, 7, 800, 600);
        assert_eq!(desktop.picture_size(&bot, 7), Some((800, 600)));
        assert_eq!(desktop.picture_size(&bot, 8), None);
    }
}
