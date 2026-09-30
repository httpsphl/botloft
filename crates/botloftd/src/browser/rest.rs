//! A browser nobody is using rests (spec 21.2): its window is minimized, so
//! its pages stop working as in any window out of sight, and it comes back
//! the moment the bot or the owner needs it. Nothing in the pages is lost.

use std::sync::Arc;

use botloft_core::ids::BotId;
use botloft_core::protocol::BrowserFrame;
use serde_json::{Value, json};
use tokio::sync::broadcast;
use tracing::debug;

use super::session::Session;
use super::{Browsers, Slots, lock, update};
use crate::clock::Clock;
use crate::state::Event;

impl Session {
    pub fn is_resting(&self) -> bool {
        self.lock().resting
    }

    /// Minimizes every window of the browser, or brings them back.
    async fn show(&self, shown: bool) {
        let tabs: Vec<String> = {
            let tabs = self.lock();
            tabs.list.iter().map(|tab| tab.target.clone()).collect()
        };
        let mut windows: Vec<Value> = Vec::new();
        for tab in tabs {
            let params = json!({ "targetId": tab });
            // A tab that closed meanwhile has no window to look for.
            if let Ok(found) = self
                .cdp
                .call(None, "Browser.getWindowForTarget", params)
                .await
                && !windows.contains(&found["windowId"])
            {
                windows.push(found["windowId"].clone());
            }
        }
        let state = if shown { "normal" } else { "minimized" };
        for window in windows {
            let params = json!({ "windowId": window, "bounds": { "windowState": state } });
            if let Err(err) = self.cdp.call(None, "Browser.setWindowBounds", params).await {
                debug!("browser: changing a window to {state}: {err}");
            }
        }
    }
}

/// Brings a resting browser back before something uses it, with frames for
/// whoever watches.
pub(super) async fn wake(
    session: &Session,
    slots: &Slots,
    events: &broadcast::Sender<Event>,
    clock: &dyn Clock,
    bot: &BotId,
) {
    let _turn = session.syncing.lock().await;
    if !session.is_resting() {
        return;
    }
    session.show(true).await;
    let watched = {
        let mut tabs = session.lock();
        tabs.resting = false;
        tabs.active()
            .filter(|_| tabs.watching)
            .map(|tab| tab.session.clone())
    };
    if let Some(page) = watched {
        session.cast(&page, false).await;
        session.cast(&page, true).await;
    }
    update(slots, events, clock, bot, |state| state.resting = false);
    debug!(bot = %bot, "browser: awake");
}

impl Browsers {
    /// Wakes the bot's browser if it rests.
    pub(super) async fn wake(&self, bot: &BotId) {
        if let Some(session) = self.running(bot) {
            wake(
                &session,
                &self.slots,
                &self.events,
                self.clock.as_ref(),
                bot,
            )
            .await;
        }
    }

    /// Puts the bot's browser to rest, unless a tool or the owner is using
    /// it. The picture of its page stays for whoever looks meanwhile.
    pub fn rest(&self, bot: &BotId) {
        let found = lock(&self.slots).get(bot).and_then(|slot| {
            let (_, session) = slot.session.as_ref()?;
            if slot.held.borrow().is_some() || session.is_resting() || session.is_closed() {
                return None;
            }
            let turn = Arc::clone(&slot.calls).try_lock_owned().ok()?;
            Some((Arc::clone(session), turn, slot.frames.clone()))
        });
        let Some((session, turn, frames)) = found else {
            return;
        };
        let (slots, events, clock) = (
            Arc::clone(&self.slots),
            self.events.clone(),
            Arc::clone(&self.clock),
        );
        let bot = bot.clone();
        tokio::spawn(async move {
            // No tool starts while the browser goes to rest.
            let _turn = turn;
            let _one = session.syncing.lock().await;
            let held = lock(&slots)
                .get(&bot)
                .is_none_or(|slot| slot.held.borrow().is_some());
            if held || session.is_resting() {
                return;
            }
            if let Ok(data) = session.screenshot().await {
                let page = session.viewport();
                frames.send_replace(Some(Arc::new(BrowserFrame {
                    bot_id: bot.clone(),
                    data,
                    width: page.width,
                    height: page.height,
                })));
            }
            session.show(false).await;
            session.lock().resting = true;
            update(&slots, &events, clock.as_ref(), &bot, |state| {
                state.resting = true;
            });
            debug!(bot = %bot, "browser: resting");
        });
    }
}
