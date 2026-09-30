//! The owner's hands in a bot's browser (spec 21.10): taking it and giving
//! it back, what the owner does reaching the browser in order, the bot's
//! tools waiting meanwhile, and the bot asking the owner for help.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use botloft_core::ids::{ApprovalId, BotId};
use botloft_core::protocol::{BrowserControl, BrowserInput};
use tokio::sync::{broadcast, mpsc};
use tracing::debug;

use super::input::fits;
use super::moves::{Move, feed};
use super::rest::wake;
use super::session::Session;
use super::{Browsers, Slots, Viewport, lock, sites, update};
use crate::clock::Clock;
use crate::state::Event;

/// Longest a bot's tool waits for the owner to give the browser back.
pub const OWNER_WAIT: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TakeError {
    #[error("the browser is not open")]
    Closed,
    #[error("someone else is using this browser")]
    Taken,
}

/// Why something the owner did was not done in the browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InputError {
    #[error("that event does not fit the page")]
    Invalid,
    #[error("that is not a web address")]
    Address,
    #[error("that tab is not open anymore")]
    NoTab,
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
    session: Arc<Session>,
    moves: mpsc::UnboundedSender<Move>,
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
            session.owner_took();
            slot.held.send_replace(Some(hold));
        }
        self.set_state(bot, |state| state.control = BrowserControl::Owner);
        debug!(bot = %bot, "browser: the owner took it");
        let (moves, queue) = mpsc::unbounded_channel();
        let (slots, events, clock) = (
            Arc::clone(&self.slots),
            self.events.clone(),
            Arc::clone(&self.clock),
        );
        let (page, owner) = (Arc::clone(&session), bot.clone());
        tokio::spawn(async move {
            // A browser at rest wakes for the owner's hands.
            wake(&page, &slots, &events, clock.as_ref(), &owner).await;
            feed(page, queue).await;
        });
        Ok(Hands {
            slots: Arc::clone(&self.slots),
            events: self.events.clone(),
            clock: Arc::clone(&self.clock),
            bot: bot.clone(),
            hold,
            session,
            moves,
        })
    }

    /// Waits while the owner holds the bot's browser, and for what they did
    /// last to reach it. `false` if they still hold it after `limit`.
    pub async fn wait_for_owner(&self, bot: &BotId, limit: Duration) -> bool {
        let mut held = {
            let mut slots = lock(&self.slots);
            self.slot(&mut slots, bot).held.subscribe()
        };
        let given_back = async {
            if held.wait_for(Option::is_none).await.is_err() {
                return false;
            }
            if let Some(session) = self.running(bot) {
                session.owner_settled().await;
            }
            true
        };
        tokio::time::timeout(limit, given_back)
            .await
            .unwrap_or(false)
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
    /// The page's size, while the owner still has the browser. What they
    /// do counts as using it.
    fn page(&self) -> Result<Viewport, InputError> {
        lock(&self.slots)
            .get_mut(&self.bot)
            .filter(|slot| *slot.held.borrow() == Some(self.hold))
            .map(|slot| {
                slot.used = Instant::now();
                slot.viewport
            })
            .ok_or(InputError::NotHeld)
    }

    /// Queues one thing the owner did, after the ones before.
    fn make(&self, step: Move) -> Result<(), InputError> {
        self.session.owner_moves.send_modify(|left| *left += 1);
        self.moves.send(step).map_err(|_| {
            self.session
                .owner_moves
                .send_modify(|left| *left = left.saturating_sub(1));
            InputError::Closed
        })
    }

    /// Sends one of the owner's events to the page.
    pub fn send(&self, input: BrowserInput) -> Result<(), InputError> {
        if !fits(&input, self.page()?) {
            return Err(InputError::Invalid);
        }
        self.make(Move::Input(input))
    }

    /// Opens a blank tab, which becomes the active one.
    pub fn new_tab(&self) -> Result<(), InputError> {
        self.page()?;
        self.make(Move::NewTab)
    }

    /// Makes the tab `id` the active one.
    pub fn switch_tab(&self, id: &str) -> Result<(), InputError> {
        self.page()?;
        if !self.session.has_tab(id) {
            return Err(InputError::NoTab);
        }
        self.make(Move::SwitchTab(id.to_owned()))
    }

    /// Takes the active tab to the web address the owner typed.
    pub fn open(&self, address: &str) -> Result<(), InputError> {
        self.page()?;
        let url = sites::typed(address).ok_or(InputError::Address)?;
        self.make(Move::Open(url))
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
