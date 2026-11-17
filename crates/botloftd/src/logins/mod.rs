//! Shared site logins (spec 35): the owner signs in to a site once, in a
//! window of the logins profile, and chooses which agents use that login.
//! Each chosen agent gets a copy of the site's cookies in its own browser;
//! nothing comes from the owner's own browser.

mod rules;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    LoginList, LoginsChanged, LoginsOpenParams, LoginsOpenResult, LoginsShareParams,
    LoginsSiteParams, SiteLogin,
};
use botloft_store::Store;
use serde_json::Value;
use tracing::{debug, warn};

use crate::browser::{BrowserError, CookieChange, cookies_of_site, sites};
use crate::service::{ApiError, ApiResult, bots, permissions};
use crate::state::{Daemon, Event};

pub use rules::write_rules;

const BUSY: &str = "a login window is open, or a login is being copied: try again when it ends";

/// The logins profile in use: its window, and one use of it at a time.
#[derive(Default)]
pub struct Logins {
    /// The site of the window open for the owner to sign in.
    window: Mutex<Option<String>>,
    profile: Arc<tokio::sync::Mutex<()>>,
}

impl Logins {
    fn window(&self) -> Option<String> {
        self.window
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn set_window(&self, site: Option<String>) {
        *self
            .window
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = site;
    }

    fn take(&self) -> ApiResult<tokio::sync::OwnedMutexGuard<()>> {
        Arc::clone(&self.profile)
            .try_lock_owned()
            .map_err(|_| ApiError::Conflict(BUSY.to_owned()))
    }
}

/// The service runs on the blocking pool, where waiting for a browser fits.
fn run<T>(work: impl Future<Output = T>) -> T {
    tokio::runtime::Handle::current().block_on(work)
}

fn browser_error(err: BrowserError) -> ApiError {
    ApiError::Conflict(err.to_string())
}

/// `logins.list`.
pub fn list(daemon: &Daemon) -> ApiResult<LoginList> {
    list_from(daemon, &daemon.store())
}

fn list_from(daemon: &Daemon, store: &Store) -> ApiResult<LoginList> {
    let mut logins = Vec::new();
    for record in store.site_logins()? {
        logins.push(SiteLogin {
            bot_ids: store.site_login_bots(&record.site)?,
            site: record.site,
            signed_at: record.signed_at,
        });
    }
    Ok(LoginList {
        logins,
        window: daemon.logins.window(),
    })
}

fn changed(daemon: &Daemon, closed: Option<String>, found: Option<bool>) {
    let list = list(daemon);
    match list {
        Ok(list) => daemon.emit(Event::LoginsChanged(LoginsChanged {
            list,
            closed,
            found,
        })),
        Err(err) => warn!("logins: could not read the list: {err}"),
    }
}

/// The site and the page of what the owner typed: "reddit.com" or a full
/// address, `http` or `https` only.
fn address(text: &str) -> ApiResult<(String, String)> {
    let text = text.trim();
    let url = if text.contains("://") {
        text.to_owned()
    } else {
        format!("https://{text}")
    };
    sites::site_of(&url)
        .map(|site| (site, url))
        .ok_or_else(|| ApiError::validation(format!("not a web address: {text:?}")))
}

/// `logins.open`: opens the logins profile in a window at the address. When
/// the owner closes it, the login is saved if the site left cookies, and
/// `logins.changed` says so.
pub fn open(daemon: &Arc<Daemon>, params: LoginsOpenParams) -> ApiResult<LoginsOpenResult> {
    let (site, url) = address(&params.address)?;
    let hold = daemon.logins.take()?;
    let (mut child, job) = daemon
        .browsers
        .open_logins_window(&url)
        .map_err(browser_error)?;
    daemon.logins.set_window(Some(site.clone()));
    changed(daemon, None, None);
    let (ended, closed) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = ended.send(());
    });
    let daemon = Arc::clone(daemon);
    let at = site.clone();
    tokio::runtime::Handle::current().spawn(async move {
        let _ = closed.await;
        // What the window left behind goes with it.
        drop(job);
        let found = match after_window(&daemon, &at).await {
            Ok(found) => found,
            Err(err) => {
                warn!("logins: could not read the login just made: {err}");
                false
            }
        };
        daemon.logins.set_window(None);
        drop(hold);
        changed(&daemon, Some(at), Some(found));
    });
    Ok(LoginsOpenResult { site })
}

/// What `logins.open` does once the owner closed its window: saves the
/// login of `site` when the profile has cookies of it, and copies it again
/// to every agent that has it. Tests call it after writing the cookies.
pub async fn signed_in(daemon: &Daemon, site: &str) -> ApiResult<bool> {
    let _hold = daemon.logins.take()?;
    after_window(daemon, site).await
}

async fn after_window(daemon: &Daemon, site: &str) -> ApiResult<bool> {
    let cookies = logins_cookies(daemon, site).await?;
    if cookies.is_empty() {
        return Ok(false);
    }
    daemon
        .store()
        .save_site_login(site, daemon.clock.now_ms())?;
    let bots = daemon.store().site_login_bots(site)?;
    for bot in bots {
        if let Err(err) = change(daemon, &bot, CookieChange::Set(cookies.clone())).await {
            warn!(bot = %bot, "logins: could not copy a login again: {err}");
        }
    }
    Ok(true)
}

async fn logins_cookies(daemon: &Daemon, site: &str) -> ApiResult<Vec<Value>> {
    let profile = daemon.browsers.logins_profile();
    let all = daemon
        .browsers
        .profile_cookies(&profile, None)
        .await
        .map_err(browser_error)?;
    Ok(cookies_of_site(&all, site))
}

fn downloads(daemon: &Daemon, bot: &BotId) -> ApiResult<PathBuf> {
    let store = daemon.store();
    let (crew, record) = bots::active(&store, bot)?;
    Ok(daemon
        .paths
        .bot_workspace(&crew.slug, &record.slug)
        .join("downloads"))
}

async fn change(daemon: &Daemon, bot: &BotId, change: CookieChange) -> ApiResult<()> {
    let downloads = downloads(daemon, bot)?;
    daemon
        .browsers
        .agent_cookies(bot, &downloads, change)
        .await
        .map_err(browser_error)
}

fn saved(daemon: &Daemon, site: &str) -> ApiResult<String> {
    let site = site.trim().to_ascii_lowercase();
    if daemon
        .store()
        .site_logins()?
        .iter()
        .any(|login| login.site == site)
    {
        Ok(site)
    } else {
        Err(ApiError::NotFound(format!("login {site}")))
    }
}

/// No agent whose browser is open in a window: its profile is in use.
fn none_in_window(daemon: &Daemon, bots: &[BotId]) -> ApiResult<()> {
    let store = daemon.store();
    for bot in bots {
        if daemon.browsers.in_window(bot) {
            let name = store
                .bot(bot)?
                .map(|record| record.name)
                .unwrap_or_default();
            return Err(ApiError::Conflict(format!(
                "{name}'s browser is open in a window: close it first"
            )));
        }
    }
    Ok(())
}

/// `logins.share`: the agents in `botIds` have the login, the others lose
/// it (spec 35.2).
pub fn share(daemon: &Daemon, params: LoginsShareParams) -> ApiResult<SiteLogin> {
    run(share_now(daemon, params))
}

async fn share_now(daemon: &Daemon, params: LoginsShareParams) -> ApiResult<SiteLogin> {
    let site = saved(daemon, &params.site)?;
    let had: Vec<BotId> = daemon.store().site_login_bots(&site)?;
    let mut wanted: Vec<BotId> = Vec::new();
    {
        let store = daemon.store();
        for bot in params.bot_ids {
            bots::active(&store, &bot)?;
            if !wanted.contains(&bot) {
                wanted.push(bot);
            }
        }
    }
    let kept: HashSet<&BotId> = wanted.iter().collect();
    let gets: Vec<BotId> = wanted
        .iter()
        .filter(|bot| !had.contains(bot))
        .cloned()
        .collect();
    let loses: Vec<BotId> = had
        .iter()
        .filter(|bot| !kept.contains(bot))
        .cloned()
        .collect();
    none_in_window(daemon, &[gets.clone(), loses.clone()].concat())?;
    let _hold = daemon.logins.take()?;
    if !gets.is_empty() {
        let cookies = logins_cookies(daemon, &site).await?;
        if cookies.is_empty() {
            return Err(ApiError::Conflict(format!(
                "the login to {site} is gone: sign in again"
            )));
        }
        for bot in &gets {
            change(daemon, bot, CookieChange::Set(cookies.clone())).await?;
            let now = daemon.clock.now_ms();
            daemon.store().give_site_login(bot, &site, now)?;
            daemon.store().allow_browser_site(bot, &site, now)?;
            permissions::sites_changed(daemon, bot)?;
            debug!(bot = %bot, "logins: gave a login");
        }
    }
    for bot in &loses {
        change(daemon, bot, CookieChange::Clear(site.clone())).await?;
        daemon.store().take_site_login(bot, &site)?;
        debug!(bot = %bot, "logins: took a login back");
    }
    for bot in gets.iter().chain(&loses) {
        rules::refresh(daemon, bot);
    }
    changed(daemon, None, None);
    list(daemon)?
        .logins
        .into_iter()
        .find(|login| login.site == site)
        .ok_or_else(|| ApiError::NotFound(format!("login {site}")))
}

/// `logins.remove`: every agent that had the login is signed out of the
/// site, the logins profile too, and the login is forgotten.
pub fn remove(daemon: &Daemon, params: LoginsSiteParams) -> ApiResult<LoginList> {
    run(remove_now(daemon, params))
}

async fn remove_now(daemon: &Daemon, params: LoginsSiteParams) -> ApiResult<LoginList> {
    let site = saved(daemon, &params.site)?;
    let had = daemon.store().site_login_bots(&site)?;
    none_in_window(daemon, &had)?;
    let _hold = daemon.logins.take()?;
    for bot in &had {
        change(daemon, bot, CookieChange::Clear(site.clone())).await?;
    }
    let profile = daemon.browsers.logins_profile();
    daemon
        .browsers
        .profile_cookies(&profile, Some(CookieChange::Clear(site.clone())))
        .await
        .map_err(browser_error)?;
    daemon.store().delete_site_login(&site)?;
    for bot in &had {
        rules::refresh(daemon, bot);
    }
    changed(daemon, None, None);
    list(daemon)
}

/// The agent was archived and its browser profile deleted: its logins go
/// with it (spec 35.2). `store` is the caller's, already held.
pub fn forget_bot(daemon: &Daemon, store: &Store, bot: &BotId) {
    match store.take_site_logins_of(bot) {
        Ok(0) => {}
        Ok(_) => match list_from(daemon, store) {
            Ok(list) => daemon.emit(Event::LoginsChanged(LoginsChanged {
                list,
                closed: None,
                found: None,
            })),
            Err(err) => warn!("logins: could not read the list: {err}"),
        },
        Err(err) => warn!(bot = %bot, "logins: could not forget an archived agent's logins: {err}"),
    }
}
