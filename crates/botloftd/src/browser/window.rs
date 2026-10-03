//! The bot's browser in a window of its own (spec 21.11): to sign in where
//! a site refuses a browser a program drives, the owner opens the bot's
//! profile in a common browser window, with no DevTools. The bot's tools
//! wait until the owner closes it; the logins stay in the profile.

use std::process::Child;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BrowserControl, BrowserState};
use tokio::sync::{broadcast, oneshot};
use tracing::{debug, warn};

use super::{BrowserError, Browsers, Slots, launch, lock, program, sites, update};
use crate::clock::Clock;
use crate::platform::ProcessJob;
use crate::state::Event;

/// The window the owner has open, and their hold on the browser meanwhile.
pub(super) struct Window {
    pub(super) hold: u64,
    job: ProcessJob,
}

impl Window {
    pub(super) fn kill(&self) {
        let _ = self.job.terminate();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("the browser is already open in a window")]
    Open,
    #[error(transparent)]
    Browser(#[from] BrowserError),
}

impl Browsers {
    /// Opens the bot's browser in a window of its own, on the page the bot
    /// was on. The browser without a window closes first: one profile, one
    /// browser. The receiver hears when the owner closes the window.
    pub fn open_window(&self, bot: &BotId) -> Result<oneshot::Receiver<()>, WindowError> {
        let program = program::find(&self.settings.path).ok_or(BrowserError::NotFound)?;
        let hold = self.holds.fetch_add(1, Ordering::Relaxed);
        let (session, url) = {
            let mut slots = lock(&self.slots);
            let slot = self.slot(&mut slots, bot);
            if slot.window.is_some() {
                return Err(WindowError::Open);
            }
            // The bot's tools wait from now on, as for the owner's hands.
            slot.held.send_replace(Some(hold));
            let url = slot
                .state
                .url
                .clone()
                .filter(|url| sites::site_of(url).is_some());
            (slot.session.take(), url)
        };
        if let Some((_, session)) = session {
            // Gone before the window starts: Edge hands a profile in use to
            // the browser that has it, and the new one ends at once.
            session.close();
        }
        let profile = self.profiles.join(bot.as_str());
        let started = launch::window(&program, &profile, url.as_deref());
        let (child, job) = match started {
            Ok(started) => started,
            Err(err) => {
                if let Some(slot) = lock(&self.slots).get_mut(bot)
                    && *slot.held.borrow() == Some(hold)
                {
                    slot.held.send_replace(None);
                }
                self.set_state(bot, |state| closed(state, false));
                return Err(err.into());
            }
        };
        if let Some(slot) = lock(&self.slots).get_mut(bot) {
            slot.window = Some(Window { hold, job });
        }
        self.set_state(bot, |state| closed(state, true));
        debug!(bot = %bot, "browser: open in a window");
        Ok(wait(
            child,
            Arc::clone(&self.slots),
            self.events.clone(),
            Arc::clone(&self.clock),
            bot.clone(),
            hold,
        ))
    }
}

/// No browser the daemon drives; the bot's request for help stays.
fn closed(state: &mut BrowserState, window: bool) {
    let ask = state.ask.take();
    *state = BrowserState::closed(state.bot_id.clone(), state.updated_at);
    state.ask = ask;
    state.window = window;
    if window {
        state.control = BrowserControl::Owner;
    }
}

/// Waits for the window to close, then gives the browser back to the bot.
fn wait(
    mut child: Child,
    slots: Slots,
    events: broadcast::Sender<Event>,
    clock: Arc<dyn Clock>,
    bot: BotId,
    hold: u64,
) -> oneshot::Receiver<()> {
    let (closed, heard) = oneshot::channel();
    let spawned = std::thread::Builder::new()
        .name("browser-window".into())
        .spawn(move || {
            if let Err(err) = child.wait() {
                warn!("browser: waiting for the window: {err}");
            }
            let mine = lock(&slots).get_mut(&bot).is_some_and(|slot| {
                let mine = slot.window.as_ref().is_some_and(|open| open.hold == hold);
                if mine {
                    slot.window = None;
                    if *slot.held.borrow() == Some(hold) {
                        slot.held.send_replace(None);
                    }
                }
                mine
            });
            if mine {
                update(&slots, &events, clock.as_ref(), &bot, |state| {
                    state.window = false;
                    state.control = BrowserControl::Bot;
                });
                debug!(bot = %bot, "browser: the window closed");
            }
            let _ = closed.send(());
        });
    if spawned.is_err() {
        warn!("browser: cannot wait for the window");
    }
    heard
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::browser::BrowserSettings;
    use crate::clock::SystemClock;

    /// A program that ends at once stands for a window the owner closed:
    /// this test binary, which refuses the browser's flags.
    fn browsers(home: &std::path::Path) -> Browsers {
        let program = std::env::current_exe().expect("test binary");
        let settings = BrowserSettings {
            path: program.display().to_string(),
            ..BrowserSettings::default()
        };
        let (events, _) = broadcast::channel(16);
        Browsers::new(settings, home, events, Arc::new(SystemClock))
    }

    #[tokio::test]
    async fn the_bot_waits_for_the_window_and_gets_the_browser_back() {
        let home = tempfile::tempdir().expect("tempdir");
        let browsers = browsers(home.path());
        let bot = BotId::generate();
        let closed = browsers.open_window(&bot).expect("window");
        closed.await.expect("closed");
        let state = browsers.view(&bot).state;
        assert!(!state.window);
        assert_eq!(state.control, BrowserControl::Bot);
        assert!(browsers.wait_for_owner(&bot, Duration::from_secs(1)).await);
        // The window is gone: another one may open.
        let again = browsers.open_window(&bot).expect("again");
        again.await.expect("closed again");
    }

    #[test]
    fn closing_keeps_what_the_bot_asked() {
        let mut state = BrowserState::closed(BotId::generate(), 0);
        state.ask = Some("Sign in".to_owned());
        state.url = Some("https://example.com/".to_owned());
        closed(&mut state, true);
        assert!(state.window);
        assert_eq!(state.control, BrowserControl::Owner);
        assert_eq!(state.ask.as_deref(), Some("Sign in"));
        assert_eq!(state.url, None);
    }
}
