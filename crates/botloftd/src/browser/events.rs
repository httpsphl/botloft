//! Keeping a browser's tabs up to date from its events (spec 21.3): new
//! tabs, closed ones, loads, requests, dialogs, downloads and frames.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use botloft_core::protocol::BrowserFrame;
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tracing::debug;

use super::cdp::CdpEvent;
use super::session::{FRAME_GAP, Hooks, MAX_TABS, PageInfo, QUICK_GAP, Session, Tab};
use super::titles::retitles;

impl Session {
    /// Sets up a new tab before it runs, and makes it the active one.
    async fn attached(&self, params: &Value, hooks: &Hooks) {
        let info = &params["targetInfo"];
        if info["type"] != "page" {
            return;
        }
        let (Some(session), Some(target)) =
            (params["sessionId"].as_str(), info["targetId"].as_str())
        else {
            return;
        };
        let (previous, watching, extra, viewport) = {
            let mut tabs = self.lock();
            let previous = tabs.active().map(|tab| tab.session.clone());
            let opened = tabs.opened;
            tabs.opened += 1;
            tabs.list.push(Tab {
                target: target.to_owned(),
                session: session.to_owned(),
                url: info["url"].as_str().unwrap_or_default().to_owned(),
                title: String::new(),
                loading: false,
                navigations: 0,
                inflight: HashSet::new(),
                network_at: Instant::now(),
                opened,
                ready: false,
            });
            let extra = (tabs.list.len() > MAX_TABS).then(|| tabs.list[0].target.clone());
            (previous, tabs.watching, extra, tabs.viewport)
        };
        // The app hears of the tab now: the setup below lets the page run,
        // and a bot's tool may read it before the setup is done.
        (hooks.changed)(self.lock().info());
        let setup = [
            ("Page.enable", json!({})),
            ("Network.enable", json!({})),
            (
                "Page.setInterceptFileChooserDialog",
                json!({ "enabled": true }),
            ),
            ("Emulation.setDeviceMetricsOverride", viewport.metrics()),
            ("Emulation.setUserAgentOverride", self.user_agent.clone()),
        ];
        // A tab opened by a page waits for us, and some of these only
        // answer once it runs: send them all, the go-ahead last, then wait.
        let mut calls: Vec<_> = setup
            .into_iter()
            .map(|(method, params)| self.cdp.call(Some(session), method, params))
            .collect();
        if params["waitingForDebugger"].as_bool() == Some(true) {
            calls.push(
                self.cdp
                    .call(Some(session), "Runtime.runIfWaitingForDebugger", json!({})),
            );
        }
        for result in futures_util::future::join_all(calls).await {
            if let Err(err) = result {
                debug!("browser: setting up a new tab: {err}");
            }
        }
        if let Some(tab) = self.lock().by_session(session) {
            tab.ready = true;
        }
        if let Some(target) = extra {
            self.cdp
                .send(None, "Target.closeTarget", json!({ "targetId": target }));
        }
        if watching {
            if let Some(previous) = previous {
                self.cast(&previous, false).await;
            }
            self.cast(session, true).await;
        }
    }

    /// A tab closed: the one before it becomes active again.
    async fn detached(&self, params: &Value) {
        let Some(session) = params["sessionId"].as_str() else {
            return;
        };
        let (was_active, now_active, watching, empty) = {
            let mut tabs = self.lock();
            let was_active = tabs.active().is_some_and(|tab| tab.session == session);
            tabs.list.retain(|tab| tab.session != session);
            let now_active = tabs
                .active()
                .map(|tab| (tab.session.clone(), tab.target.clone()));
            (was_active, now_active, tabs.watching, tabs.list.is_empty())
        };
        if empty && !self.is_closed() {
            self.cdp
                .send(None, "Target.createTarget", json!({ "url": "about:blank" }));
        }
        if was_active && let Some((active, target)) = now_active {
            // The browser may have picked another tab to show.
            self.cdp
                .send(None, "Target.activateTarget", json!({ "targetId": target }));
            if watching {
                self.cast(&active, true).await;
            }
        }
    }

    async fn handle(&self, event: &CdpEvent, hooks: &Hooks) {
        let params = &event.params;
        let Some(session) = event.session.as_deref() else {
            match event.method.as_str() {
                "Target.attachedToTarget" => self.attached(params, hooks).await,
                "Target.detachedFromTarget" => self.detached(params).await,
                "Target.targetInfoChanged" => self.info_changed(&params["targetInfo"]),
                "Browser.downloadWillBegin" => {
                    if let (Some(guid), Some(name)) = (
                        params["guid"].as_str(),
                        params["suggestedFilename"].as_str(),
                    ) {
                        self.lock()
                            .downloads
                            .insert(guid.to_owned(), name.to_owned());
                    }
                }
                "Browser.downloadProgress" => self.download(params),
                _ => {}
            }
            return;
        };
        match event.method.as_str() {
            "Page.screencastFrame" => self.frame(session, params, hooks),
            "Page.javascriptDialogOpening" => {
                self.cdp.send(
                    Some(session),
                    "Page.handleJavaScriptDialog",
                    json!({ "accept": true }),
                );
                let message = params["message"].as_str().unwrap_or_default();
                let kind = params["type"].as_str().unwrap_or("alert");
                self.lock().notes.push(format!(
                    "The page showed a dialog ({kind}) and it was accepted: \"{}\"",
                    botloft_core::chat::one_line(message, 300)
                ));
            }
            "Page.fileChooserOpened" => self.lock().notes.push(
                "The page asked for a file to upload. Uploading files from the browser is not \
                 possible yet; tell the owner if you need it."
                    .to_owned(),
            ),
            _ => self.page_event(session, &event.method, params),
        }
    }

    fn info_changed(&self, info: &Value) {
        let Some(target) = info["targetId"].as_str() else {
            return;
        };
        let mut tabs = self.lock();
        if let Some(tab) = tabs.list.iter_mut().find(|tab| tab.target == target) {
            // The title here is the address again, never the page's
            // (`titles.rs`).
            if let Some(url) = info["url"].as_str() {
                url.clone_into(&mut tab.url);
            }
        }
    }

    fn download(&self, params: &Value) {
        let state = params["state"].as_str().unwrap_or_default();
        if state != "completed" && state != "canceled" {
            return;
        }
        let mut tabs = self.lock();
        let guid = params["guid"].as_str().unwrap_or_default();
        let name = tabs.downloads.remove(guid).unwrap_or_default();
        let note = if state == "completed" {
            format!("Downloaded {name} into the downloads folder of your folder.")
        } else {
            format!("The download of {name} was canceled.")
        };
        tabs.notes.push(note);
    }

    fn frame(&self, session: &str, params: &Value, hooks: &Hooks) {
        let (active, viewport, gap) = {
            let tabs = self.lock();
            let active = tabs.active().is_some_and(|tab| tab.session == session);
            let gap = if tabs.quick { QUICK_GAP } else { FRAME_GAP };
            (active, tabs.viewport, gap)
        };
        let size = |key: &str, fallback: u32| {
            params["metadata"][key]
                .as_f64()
                .map_or(fallback, |value| value.round() as u32)
        };
        let (width, height) = (
            size("deviceWidth", viewport.width),
            size("deviceHeight", viewport.height),
        );
        if active && let Some(data) = params["data"].as_str() {
            hooks.frames.send_replace(Some(Arc::new(BrowserFrame {
                bot_id: hooks.bot.clone(),
                data: data.to_owned(),
                width,
                height,
            })));
        }
        let cdp = self.cdp.clone();
        let session = session.to_owned();
        let ack = json!({ "sessionId": params["sessionId"] });
        tokio::spawn(async move {
            tokio::time::sleep(gap).await;
            cdp.send(Some(&session), "Page.screencastFrameAck", ack);
        });
    }

    fn page_event(&self, session: &str, method: &str, params: &Value) {
        let mut tabs = self.lock();
        let Some(tab) = tabs.by_session(session) else {
            return;
        };
        let main = params["frameId"].as_str() == Some(tab.target.as_str());
        match method {
            "Page.frameStartedLoading" if main => {
                tab.loading = true;
                tab.navigations += 1;
            }
            "Page.frameStoppedLoading" if main => tab.loading = false,
            "Page.frameNavigated" if params["frame"]["parentId"].is_null() => {
                if let Some(url) = params["frame"]["url"].as_str() {
                    let fragment = params["frame"]["urlFragment"].as_str().unwrap_or_default();
                    tab.url = format!("{url}{fragment}");
                }
                // The new page's title is read once it loads.
                tab.title.clear();
                tab.inflight.clear();
            }
            "Page.navigatedWithinDocument" if main => {
                if let Some(url) = params["url"].as_str() {
                    url.clone_into(&mut tab.url);
                }
            }
            "Network.requestWillBeSent" => {
                let long_lived =
                    matches!(params["type"].as_str(), Some("WebSocket" | "EventSource"));
                if !long_lived && let Some(id) = params["requestId"].as_str() {
                    tab.inflight.insert(id.to_owned());
                    tab.network_at = Instant::now();
                }
            }
            "Network.loadingFinished" | "Network.loadingFailed" => {
                if let Some(id) = params["requestId"].as_str()
                    && tab.inflight.remove(id)
                {
                    tab.network_at = Instant::now();
                }
            }
            _ => {}
        }
    }
}

/// Reads the browser's events until it closes.
pub(super) async fn pump(
    session: Arc<Session>,
    mut events: mpsc::UnboundedReceiver<CdpEvent>,
    hooks: Hooks,
) {
    let mut last = PageInfo::default();
    loop {
        let handled = tokio::select! {
            event = events.recv() => {
                let Some(event) = event else { break };
                session.handle(&event, &hooks).await;
                if let Some(page) = event.session.filter(|_| retitles(&event.method)) {
                    let session = Arc::clone(&session);
                    tokio::spawn(async move { session.retitle(&page).await });
                }
                true
            }
            // The owner moved between tabs: no event says so.
            () = session.moved.notified() => false,
        };
        let info = session.lock().info();
        if info != last {
            (hooks.changed)(info.clone());
            last = info;
        }
        // Only now: a tool that waits for this event answers after the
        // app's state shows it, so the tab list is never behind the bot.
        if handled {
            session.changed.notify_waiters();
        }
    }
    debug!("browser: closed");
    session.close();
    (hooks.closed)();
}
