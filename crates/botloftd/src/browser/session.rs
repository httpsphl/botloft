//! One bot's running browser (spec 21.2, 21.3): the process, its DevTools
//! connection and its tabs, kept up to date from the browser's events.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use botloft_core::ids::BotId;
use botloft_core::protocol::BrowserFrame;
use serde_json::json;
use tokio::sync::{Notify, watch};
use tracing::debug;

use super::BrowserError;
use super::cdp::Cdp;
use super::events::pump;
use super::launch::{self, BrowserProcess};
use super::viewport::{MAX_HEIGHT, Viewport, WIDTH};

/// Most tabs a browser keeps; the oldest inactive one closes past this.
pub(super) const MAX_TABS: usize = 6;
/// Frames are confirmed after this, so the browser sends ~15 a second.
pub(super) const FRAME_GAP: Duration = Duration::from_millis(66);
/// How long the first tab may take to show up.
const FIRST_TAB_TIMEOUT: Duration = Duration::from_secs(5);

/// What the app shows about the active tab.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageInfo {
    pub url: Option<String>,
    pub title: Option<String>,
    pub loading: bool,
    pub tabs: u32,
}

/// How a session reports back to the hub.
pub struct Hooks {
    pub bot: BotId,
    pub frames: watch::Sender<Option<Arc<BrowserFrame>>>,
    pub changed: Box<dyn Fn(PageInfo) + Send + Sync>,
    /// The browser went away; called once.
    pub closed: Box<dyn Fn() + Send + Sync>,
}

pub struct Tab {
    pub target: String,
    pub session: String,
    pub url: String,
    pub title: String,
    pub loading: bool,
    /// Main-frame loads started, to tell whether an action navigated.
    pub navigations: u64,
    pub inflight: HashSet<String>,
    /// The last request that started or ended.
    pub network_at: Instant,
}

#[derive(Default)]
pub struct Tabs {
    /// In the order they became active; the last one is.
    pub list: Vec<Tab>,
    /// What the bot should read with the next result: dialogs, downloads.
    pub notes: Vec<String>,
    pub(super) downloads: HashMap<String, String>,
    pub watching: bool,
    pub closed: bool,
    pub viewport: Viewport,
}

impl Tabs {
    pub fn active(&self) -> Option<&Tab> {
        self.list.last()
    }

    pub(super) fn by_session(&mut self, session: &str) -> Option<&mut Tab> {
        self.list.iter_mut().find(|tab| tab.session == session)
    }

    pub(super) fn info(&self) -> PageInfo {
        let tab = self.active();
        PageInfo {
            url: tab.map(|tab| tab.url.clone()).filter(|url| !url.is_empty()),
            title: tab
                .map(|tab| tab.title.clone())
                .filter(|title| !title.is_empty()),
            loading: tab.is_some_and(|tab| tab.loading),
            tabs: u32::try_from(self.list.len()).unwrap_or(u32::MAX),
        }
    }
}

pub struct Session {
    pub cdp: Cdp,
    pub tabs: Mutex<Tabs>,
    /// Wakes whoever waits for the page after any event.
    pub changed: Notify,
    /// The next element ref, shared by every page of this browser.
    pub next_ref: AtomicU64,
    process: Mutex<Option<BrowserProcess>>,
    pub(super) user_agent: String,
    /// One change of what the app wants at a time (`watch.rs`).
    pub(super) syncing: tokio::sync::Mutex<()>,
}

impl Session {
    /// Starts the browser, its pages `viewport` in size, and waits for its
    /// first tab.
    pub async fn start(
        program: &Path,
        profile: &Path,
        downloads: &Path,
        viewport: Viewport,
        hooks: Hooks,
    ) -> Result<Arc<Self>, BrowserError> {
        let (process, url) = launch::launch(program, profile).await?;
        let (cdp, events) = Cdp::connect(&url).await?;
        let version = cdp.call(None, "Browser.getVersion", json!({})).await?;
        let user_agent = version["userAgent"]
            .as_str()
            .unwrap_or_default()
            .replace("HeadlessChrome", "Chrome");
        let session = Arc::new(Self {
            cdp,
            tabs: Mutex::new(Tabs {
                viewport,
                ..Tabs::default()
            }),
            changed: Notify::new(),
            next_ref: AtomicU64::new(1),
            process: Mutex::new(Some(process)),
            user_agent,
            syncing: tokio::sync::Mutex::new(()),
        });
        tokio::spawn(pump(Arc::clone(&session), events, hooks));

        std::fs::create_dir_all(downloads)?;
        let cdp = &session.cdp;
        cdp.call(
            None,
            "Target.setDiscoverTargets",
            json!({ "discover": true }),
        )
        .await?;
        cdp.call(
            None,
            "Browser.setDownloadBehavior",
            json!({ "behavior": "allow", "downloadPath": downloads, "eventsEnabled": true }),
        )
        .await?;
        cdp.call(
            None,
            "Target.setAutoAttach",
            json!({
                "autoAttach": true,
                "waitForDebuggerOnStart": true,
                "flatten": true,
                "filter": [{ "type": "page", "exclude": false }, { "exclude": true }],
            }),
        )
        .await?;
        let deadline = Instant::now() + FIRST_TAB_TIMEOUT;
        let ready = loop {
            let notified = session.changed.notified();
            if session.lock().active().is_some() {
                break true;
            }
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() || tokio::time::timeout(left, notified).await.is_err() {
                break false;
            }
        };
        if !ready {
            session.close();
            return Err(BrowserError::Start("the browser opened no page".to_owned()));
        }
        Ok(session)
    }

    pub fn lock(&self) -> MutexGuard<'_, Tabs> {
        self.tabs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn is_closed(&self) -> bool {
        self.cdp.is_closed() || self.lock().closed
    }

    /// Kills the browser. Its events end, and the hub hears it closed.
    pub fn close(&self) {
        self.lock().closed = true;
        let process = self
            .process
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        drop(process);
        self.changed.notify_waiters();
    }

    /// The active tab's DevTools session and target.
    pub fn active(&self) -> Option<(String, String)> {
        self.lock()
            .active()
            .map(|tab| (tab.session.clone(), tab.target.clone()))
    }

    pub fn take_notes(&self) -> Vec<String> {
        std::mem::take(&mut self.lock().notes)
    }

    /// Starts or stops the live frames of the active tab.
    pub async fn set_watching(&self, on: bool) {
        let active = {
            let mut tabs = self.lock();
            tabs.watching = on;
            tabs.active().map(|tab| tab.session.clone())
        };
        if let Some(session) = active {
            self.cast(&session, on).await;
        }
    }

    pub(super) async fn cast(&self, session: &str, on: bool) {
        let result = if on {
            // Frames come the size of the page, however tall it is.
            let params = json!({
                "format": "jpeg", "quality": 60, "maxWidth": WIDTH, "maxHeight": MAX_HEIGHT,
            });
            self.cdp
                .call(Some(session), "Page.startScreencast", params)
                .await
        } else {
            self.cdp
                .call(Some(session), "Page.stopScreencast", json!({}))
                .await
        };
        if let Err(err) = result {
            debug!("browser: screencast {on}: {err}");
        }
    }
}
