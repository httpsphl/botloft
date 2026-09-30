//! The owner's hands in a bot's browser (spec 21.10): taking it and giving
//! it back, the owner's events reaching the page in order, the bot's tools
//! waiting meanwhile, and the bot asking the owner for help.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use botloft_core::ids::{ApprovalId, BotId};
use botloft_core::protocol::{BrowserControl, BrowserInput, MouseAction};
use tokio::sync::{broadcast, mpsc};
use tracing::debug;

use super::session::Session;
use super::{Browsers, Slots, Viewport, lock, update};
use crate::clock::Clock;
use crate::state::Event;

/// Longest a bot's tool waits for the owner to give the browser back.
pub const OWNER_WAIT: Duration = Duration::from_secs(10 * 60);
/// Longest text the owner sends at once, in characters.
const TEXT_MAX: usize = 10_000;
/// Longest key or code name.
const KEY_MAX: usize = 32;
/// Farthest one turn of the wheel scrolls, in pixels.
const WHEEL_MAX: f64 = 10_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TakeError {
    #[error("the browser is not open")]
    Closed,
    #[error("someone else is using this browser")]
    Taken,
}

/// Why an event of the owner's did not go to the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InputError {
    #[error("that event does not fit the page")]
    Invalid,
    #[error("the browser is back with the bot; take it again")]
    NotHeld,
    #[error("the browser closed")]
    Closed,
}

/// The owner's hold on a bot's browser, kept by the app connection that
/// took it. Dropping it gives the browser back to the bot.
pub struct Hands {
    slots: Slots,
    events: broadcast::Sender<Event>,
    clock: Arc<dyn Clock>,
    pub bot: BotId,
    hold: u64,
    input: mpsc::UnboundedSender<BrowserInput>,
}

impl Browsers {
    /// The owner takes the bot's browser: their events go to the page, and
    /// the bot's tools wait until they give it back.
    pub fn take(&self, bot: &BotId) -> Result<Hands, TakeError> {
        let session = self.running(bot).ok_or(TakeError::Closed)?;
        let hold = self.holds.fetch_add(1, Ordering::Relaxed);
        {
            let mut slots = lock(&self.slots);
            let slot = self.slot(&mut slots, bot);
            if slot.held.borrow().is_some() {
                return Err(TakeError::Taken);
            }
            slot.held.send_replace(Some(hold));
        }
        self.set_state(bot, |state| state.control = BrowserControl::Owner);
        debug!(bot = %bot, "browser: the owner took it");
        let (input, queue) = mpsc::unbounded_channel();
        tokio::spawn(feed(session, queue));
        Ok(Hands {
            slots: Arc::clone(&self.slots),
            events: self.events.clone(),
            clock: Arc::clone(&self.clock),
            bot: bot.clone(),
            hold,
            input,
        })
    }

    /// Waits while the owner holds the bot's browser. `false` if they still
    /// hold it after `limit`.
    pub async fn wait_for_owner(&self, bot: &BotId, limit: Duration) -> bool {
        let mut held = {
            let mut slots = lock(&self.slots);
            self.slot(&mut slots, bot).held.subscribe()
        };
        let waited = tokio::time::timeout(limit, held.wait_for(Option::is_none)).await;
        matches!(waited, Ok(Ok(_)))
    }

    /// The bot asks the owner for help: the app shows `task` until the
    /// guard drops.
    pub fn asking(&self, bot: &BotId, task: &str) -> Asking<'_> {
        self.set_state(bot, |state| state.ask = Some(task.to_owned()));
        Asking {
            browsers: self,
            bot: bot.clone(),
        }
    }

    /// The bot's open request for help, taken to answer it: the owner gave
    /// the browser back.
    pub fn take_ask(&self, bot: &BotId) -> Option<ApprovalId> {
        lock(&self.slots)
            .get_mut(bot)
            .and_then(|slot| slot.asking.take())
    }
}

/// A bot waiting for the owner's help in its browser.
pub struct Asking<'a> {
    browsers: &'a Browsers,
    bot: BotId,
}

impl Asking<'_> {
    /// The request is in the chat as `approval`.
    pub fn opened(&self, approval: &ApprovalId) {
        if let Some(slot) = lock(&self.browsers.slots).get_mut(&self.bot) {
            slot.asking = Some(approval.clone());
        }
    }
}

impl Drop for Asking<'_> {
    /// Answered or not, the browser goes back to the bot.
    fn drop(&mut self) {
        if let Some(slot) = lock(&self.browsers.slots).get_mut(&self.bot) {
            slot.asking = None;
            slot.held.send_replace(None);
        }
        self.browsers.set_state(&self.bot, |state| {
            state.ask = None;
            state.control = BrowserControl::Bot;
        });
    }
}

impl Hands {
    /// Sends one of the owner's events to the page, after the ones before.
    pub fn send(&self, input: BrowserInput) -> Result<(), InputError> {
        let page = lock(&self.slots)
            .get(&self.bot)
            .filter(|slot| *slot.held.borrow() == Some(self.hold))
            .map(|slot| slot.viewport);
        let Some(page) = page else {
            return Err(InputError::NotHeld);
        };
        if !fits(&input, page) {
            return Err(InputError::Invalid);
        }
        self.input.send(input).map_err(|_| InputError::Closed)
    }
}

impl Drop for Hands {
    fn drop(&mut self) {
        let released = lock(&self.slots).get_mut(&self.bot).is_some_and(|slot| {
            let mine = *slot.held.borrow() == Some(self.hold);
            if mine {
                slot.held.send_replace(None);
            }
            mine
        });
        if released {
            update(
                &self.slots,
                &self.events,
                self.clock.as_ref(),
                &self.bot,
                |state| state.control = BrowserControl::Bot,
            );
            debug!(bot = %self.bot, "browser: the owner gave it back");
        }
    }
}

/// Whether an event makes sense on a page of that size.
fn fits(input: &BrowserInput, page: Viewport) -> bool {
    let on_page = |x: f64, y: f64| page.contains(x, y);
    let modifiers_ok = |modifiers: u32| modifiers <= 15;
    match input {
        BrowserInput::Mouse {
            x,
            y,
            buttons,
            clicks,
            modifiers,
            ..
        } => on_page(*x, *y) && *buttons <= 7 && *clicks <= 3 && modifiers_ok(*modifiers),
        BrowserInput::Wheel {
            x,
            y,
            dx,
            dy,
            modifiers,
        } => {
            on_page(*x, *y)
                && dx.abs() <= WHEEL_MAX
                && dy.abs() <= WHEEL_MAX
                && modifiers_ok(*modifiers)
        }
        BrowserInput::Key {
            key,
            code,
            modifiers,
        } => {
            !key.is_empty()
                && key.chars().count() <= KEY_MAX
                && code.len() <= KEY_MAX
                && modifiers_ok(*modifiers)
        }
        BrowserInput::Text { text } => !text.is_empty() && text.chars().count() <= TEXT_MAX,
    }
}

/// Sends the owner's events to the page one by one, in order. Of several
/// mouse moves in a row only the newest matters.
async fn feed(session: Arc<Session>, mut queue: mpsc::UnboundedReceiver<BrowserInput>) {
    let mut next = queue.recv().await;
    while let Some(mut input) = next.take() {
        while let Ok(later) = queue.try_recv() {
            if moves(&input) && moves(&later) {
                input = later;
            } else {
                next = Some(later);
                break;
            }
        }
        if session.is_closed() {
            return;
        }
        if let Err(err) = session.owner_input(&input).await {
            debug!("browser: an event of the owner's did not reach the page: {err}");
        }
        if next.is_none() {
            next = queue.recv().await;
        }
    }
}

fn moves(input: &BrowserInput) -> bool {
    matches!(
        input,
        BrowserInput::Mouse {
            action: MouseAction::Move,
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::MouseButton;

    use super::*;

    fn mouse(x: f64, y: f64) -> BrowserInput {
        BrowserInput::Mouse {
            action: MouseAction::Down,
            x,
            y,
            button: MouseButton::Left,
            buttons: 1,
            clicks: 1,
            modifiers: 0,
        }
    }

    #[test]
    fn events_off_the_page_or_too_big_are_refused() {
        let fits = |input: &BrowserInput| super::fits(input, Viewport::default());
        assert!(fits(&mouse(640.0, 400.0)));
        assert!(fits(&mouse(1280.0, 800.0)));
        assert!(!fits(&mouse(-1.0, 10.0)));
        assert!(!fits(&mouse(f64::NAN, 10.0)));
        assert!(!fits(&mouse(10.0, 900.0)));
        // A taller page takes points further down.
        let tall = Viewport::fitting(640, 700);
        assert!(super::fits(&mouse(10.0, 900.0), tall));
        let long = BrowserInput::Text {
            text: "x".repeat(TEXT_MAX + 1),
        };
        assert!(!fits(&long));
        let empty = BrowserInput::Text {
            text: String::new(),
        };
        assert!(!fits(&empty));
        let key = BrowserInput::Key {
            key: "a".to_owned(),
            code: "KeyA".to_owned(),
            modifiers: 16,
        };
        assert!(!fits(&key));
    }
}
