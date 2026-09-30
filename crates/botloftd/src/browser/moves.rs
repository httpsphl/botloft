//! What the owner does in a bot's browser besides using the page (spec
//! 21.10): reloading it, moving between tabs, opening a new one and going
//! to an address. With the browser in their hands these go to the page one
//! by one, in order, along with their mouse and keyboard.

use std::sync::Arc;
use std::time::{Duration, Instant};

use botloft_core::ids::BotId;
use botloft_core::protocol::{BrowserInput, MouseAction};
use serde_json::json;
use tokio::sync::mpsc;
use tracing::debug;

use super::session::{Session, Tab};
use super::{BrowserError, Browsers};

/// How long a new tab may take to show up.
const NEW_TAB_TIMEOUT: Duration = Duration::from_secs(5);
/// What the bot reads after the owner reloaded its page.
const RELOADED: &str = "The owner reloaded the page.";

/// One thing the owner did with the browser in their hands.
pub(super) enum Move {
    Input(BrowserInput),
    NewTab,
    /// The tab, by its target, that becomes the active one.
    SwitchTab(String),
    /// A web address for the active tab.
    Open(String),
}

impl Browsers {
    /// Reloads the active tab of the bot's browser, in the owner's hands or
    /// not. `false` when the browser is not open.
    pub fn reload(&self, bot: &BotId) -> bool {
        let Some(session) = self.running(bot) else {
            return false;
        };
        tokio::spawn(async move {
            if let Err(err) = session.reload().await {
                debug!("browser: the page did not reload: {err}");
            }
        });
        true
    }
}

impl Session {
    /// Reloads the active tab. The bot reads so with its next result.
    async fn reload(&self) -> Result<(), BrowserError> {
        let (page, _) = self.page()?;
        {
            let mut tabs = self.lock();
            if tabs.notes.last().is_none_or(|note| note != RELOADED) {
                tabs.notes.push(RELOADED.to_owned());
            }
        }
        self.cdp.call(Some(&page), "Page.reload", json!({})).await?;
        Ok(())
    }

    pub(super) fn has_tab(&self, target: &str) -> bool {
        self.lock().list.iter().any(|tab| tab.target == target)
    }

    /// Opens a blank tab and waits for it to be the active one, set up.
    async fn new_tab(&self) -> Result<(), BrowserError> {
        let created = self
            .cdp
            .call(None, "Target.createTarget", json!({ "url": "about:blank" }))
            .await?;
        let target = created["targetId"].as_str().unwrap_or_default();
        let deadline = Instant::now() + NEW_TAB_TIMEOUT;
        loop {
            let notified = self.changed.notified();
            let open = |tab: &Tab| tab.target == target && tab.ready;
            if self.lock().active().is_some_and(open) {
                return Ok(());
            }
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() || tokio::time::timeout(left, notified).await.is_err() {
                return Err(BrowserError::Page("the new tab did not open".to_owned()));
            }
        }
    }

    /// Makes the tab `target` the active one: in front in the browser, with
    /// the live frames, and where tools and hands act.
    async fn switch_tab(&self, target: &str) -> Result<(), BrowserError> {
        let (previous, next, watching) = {
            let mut tabs = self.lock();
            let Some(index) = tabs.list.iter().position(|tab| tab.target == target) else {
                return Ok(());
            };
            if index + 1 == tabs.list.len() {
                return Ok(());
            }
            let previous = tabs.active().map(|tab| tab.session.clone());
            let tab = tabs.list.remove(index);
            let next = tab.session.clone();
            tabs.list.push(tab);
            (previous, next, tabs.watching)
        };
        self.moved.notify_one();
        let shown = self
            .cdp
            .call(None, "Target.activateTarget", json!({ "targetId": target }))
            .await;
        if watching {
            if let Some(previous) = previous {
                self.cast(&previous, false).await;
            }
            self.cast(&next, true).await;
        }
        shown?;
        Ok(())
    }

    /// Takes the active tab to `url`, without waiting for the page.
    async fn go(&self, url: &str) -> Result<(), BrowserError> {
        let (page, _) = self.page()?;
        self.cdp
            .call(Some(&page), "Page.navigate", json!({ "url": url }))
            .await?;
        Ok(())
    }

    /// The owner took the browser: remember the tab the bot was on.
    pub(super) fn owner_took(&self) {
        let mut tabs = self.lock();
        if tabs.owner_from.is_none() {
            tabs.owner_from = tabs.active().map(|tab| tab.target.clone());
        }
    }

    /// Whether the owner had the browser and left the bot on another tab
    /// than the one it was on; asked once, by the bot's next tool (spec
    /// 21.10).
    pub fn take_switched(&self) -> bool {
        let mut tabs = self.lock();
        let now = tabs.active().map(|tab| tab.target.clone());
        tabs.owner_from
            .take()
            .is_some_and(|from| now.as_ref() != Some(&from))
    }

    /// Waits until everything the owner did has reached the browser.
    pub(super) async fn owner_settled(&self) {
        let mut left = self.owner_moves.subscribe();
        let _ = left.wait_for(|left| *left == 0).await;
    }

    async fn make(&self, step: &Move) -> Result<(), BrowserError> {
        match step {
            Move::Input(input) => self.owner_input(input).await,
            Move::NewTab => self.new_tab().await,
            Move::SwitchTab(target) => self.switch_tab(target).await,
            Move::Open(url) => self.go(url).await,
        }
    }
}

/// Does what the owner did, one thing at a time, in order. Of several mouse
/// moves in a row only the newest matters. Each thing queued counts in
/// `owner_moves` until it is done or skipped.
pub(super) async fn feed(session: Arc<Session>, mut queue: mpsc::UnboundedReceiver<Move>) {
    let done = || {
        session
            .owner_moves
            .send_modify(|left| *left = left.saturating_sub(1))
    };
    let mut next = queue.recv().await;
    while let Some(mut step) = next.take() {
        while let Ok(later) = queue.try_recv() {
            if glides(&step) && glides(&later) {
                step = later;
                done();
            } else {
                next = Some(later);
                break;
            }
        }
        if !session.is_closed()
            && let Err(err) = session.make(&step).await
        {
            debug!("browser: something the owner did failed in the browser: {err}");
        }
        done();
        if next.is_none() {
            next = queue.recv().await;
        }
    }
}

fn glides(step: &Move) -> bool {
    matches!(
        step,
        Move::Input(BrowserInput::Mouse {
            action: MouseAction::Move,
            ..
        })
    )
}
