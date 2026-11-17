//! A site's cookies between the browsers Botloft runs (spec 35.2): read in
//! the logins profile, written in an agent's browser, or made to expire.
//! They pass through the DevTools only; nothing of them goes to the log.

use std::path::{Path, PathBuf};
use std::process::Child;

use botloft_core::ids::BotId;
use serde_json::{Value, json};

use super::cdp::{Cdp, CdpError};
use super::{BrowserError, Browsers, launch, lock, program};
use crate::platform::ProcessJob;

/// The profile folder the owner signs in to sites with, for the agents.
const LOGINS: &str = "logins";

/// What `Storage.setCookies` takes of what `Storage.getCookies` gives.
const KEPT: [&str; 10] = [
    "name",
    "value",
    "domain",
    "path",
    "secure",
    "httpOnly",
    "sameSite",
    "priority",
    "sourceScheme",
    "sourcePort",
];

/// The cookies of `site` in `cookies`, as `Storage.setCookies` takes them:
/// those whose domain is the site or ends with `.<site>`.
pub fn of_site(cookies: &[Value], site: &str) -> Vec<Value> {
    let suffix = format!(".{site}");
    cookies
        .iter()
        .filter(|cookie| {
            let domain = cookie["domain"].as_str().unwrap_or_default();
            let domain = domain.trim_start_matches('.').to_ascii_lowercase();
            domain == site || domain.ends_with(&suffix)
        })
        .map(|cookie| {
            let mut param = serde_json::Map::new();
            for field in KEPT {
                if let Some(value) = cookie.get(field) {
                    param.insert(field.to_owned(), value.clone());
                }
            }
            // A session cookie has no date; giving it one would keep it.
            if cookie["session"] != json!(true)
                && let Some(expires) = cookie.get("expires")
            {
                param.insert("expires".to_owned(), expires.clone());
            }
            if let Some(key) = cookie.get("partitionKey") {
                param.insert("partitionKey".to_owned(), key.clone());
            }
            Value::Object(param)
        })
        .collect()
}

/// The same cookies, gone: empty and expired.
pub fn expired(cookies: &[Value]) -> Vec<Value> {
    cookies
        .iter()
        .map(|cookie| {
            let mut cookie = cookie.clone();
            cookie["value"] = json!("");
            cookie["expires"] = json!(1);
            cookie
        })
        .collect()
}

/// What to do with a browser's cookies.
#[derive(Debug, Clone)]
pub enum CookieChange {
    /// Write these, as [`of_site`] gives them.
    Set(Vec<Value>),
    /// Make every cookie of this site expire, whatever the browser holds:
    /// a site may have renewed them since they were copied.
    Clear(String),
}

async fn apply(cdp: &Cdp, change: &CookieChange) -> Result<(), CdpError> {
    let cookies = match change {
        CookieChange::Set(cookies) => cookies.clone(),
        CookieChange::Clear(site) => {
            let all = cdp.call(None, "Storage.getCookies", json!({})).await?;
            expired(&of_site(
                all["cookies"].as_array().map_or(&[], Vec::as_slice),
                site,
            ))
        }
    };
    if !cookies.is_empty() {
        cdp.call(None, "Storage.setCookies", json!({ "cookies": cookies }))
            .await?;
    }
    Ok(())
}

impl Browsers {
    /// The profile of the shared logins, next to the agents' ones, so its
    /// DevTools port is one no page may open (spec 21.5).
    pub fn logins_profile(&self) -> PathBuf {
        self.profiles.join(LOGINS)
    }

    /// Whether the agent's browser is open in a window of its own (spec
    /// 21.11): its profile is in use, and no other browser can open it.
    pub fn in_window(&self, bot: &BotId) -> bool {
        lock(&self.slots)
            .get(bot)
            .is_some_and(|slot| slot.window.is_some())
    }

    /// Opens the logins profile in a common window at `url`, for the owner
    /// to sign in (spec 35.2). The process is the browser.
    pub fn open_logins_window(&self, url: &str) -> Result<(Child, ProcessJob), BrowserError> {
        let program = program::find(&self.settings.path).ok_or(BrowserError::NotFound)?;
        launch::window(&program, &self.logins_profile(), Some(url))
    }

    /// Opens `profile` without a window for a moment, changes its cookies
    /// if asked, and answers every cookie it has then. It closes as a
    /// person would, so the change is kept (19).
    pub async fn profile_cookies(
        &self,
        profile: &Path,
        change: Option<CookieChange>,
    ) -> Result<Vec<Value>, BrowserError> {
        let program = program::find(&self.settings.path).ok_or(BrowserError::NotFound)?;
        let (mut process, url) = launch::launch(&program, profile).await?;
        let read = async {
            let (cdp, _events) = Cdp::connect(&url).await?;
            if let Some(change) = &change {
                apply(&cdp, change).await?;
            }
            let all = cdp.call(None, "Storage.getCookies", json!({})).await?;
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                cdp.call(None, "Browser.close", json!({})),
            )
            .await;
            Ok::<_, BrowserError>(all["cookies"].as_array().cloned().unwrap_or_default())
        }
        .await;
        process.ended(std::time::Duration::from_secs(5)).await;
        process.kill();
        read
    }

    /// Changes the cookies of the agent's browser, which waits for the call
    /// before it. A closed browser opens without a window just for that,
    /// without waking the agent, and closes again as a person would.
    pub async fn agent_cookies(
        &self,
        bot: &BotId,
        downloads: &Path,
        change: CookieChange,
    ) -> Result<(), BrowserError> {
        if self.in_window(bot) {
            return Err(BrowserError::Page(
                "the agent's browser is open in a window".to_owned(),
            ));
        }
        let call = self.begin(bot).await;
        let opened = call.running().is_none();
        let session = call.session(downloads).await?;
        let written = apply(&session.cdp, &change).await;
        if opened {
            let session = lock(&self.slots)
                .get_mut(bot)
                .and_then(|slot| slot.session.take());
            if let Some((_, session)) = session {
                session.shut().await;
            }
            self.close(bot);
        }
        drop(call);
        written.map(|_| ()).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{expired, of_site};

    #[test]
    fn a_sites_cookies_are_its_own_and_its_subdomains() {
        let cookies = [
            json!({ "name": "token", "value": "a", "domain": ".reddit.com", "path": "/",
                "secure": true, "httpOnly": true, "sameSite": "None", "session": false,
                "expires": 1_900_000_000.5, "size": 6, "priority": "Medium",
                "sourceScheme": "Secure", "sourcePort": 443 }),
            json!({ "name": "s", "value": "b", "domain": "www.reddit.com", "path": "/",
                "session": true, "expires": -1 }),
            json!({ "name": "x", "value": "c", "domain": ".notreddit.com", "path": "/" }),
            json!({ "name": "MUID", "value": "d", "domain": ".bing.com", "path": "/" }),
        ];
        let mine = of_site(&cookies, "reddit.com");
        assert_eq!(mine.len(), 2);
        assert_eq!(mine[0]["expires"], json!(1_900_000_000.5));
        assert!(mine[0].get("size").is_none());
        assert_eq!(mine[0]["sourcePort"], json!(443));
        // A session cookie stays one.
        assert!(mine[1].get("expires").is_none());

        let gone = expired(&mine);
        assert_eq!(gone[1]["expires"], json!(1));
        assert_eq!(gone[0]["value"], json!(""));
        assert_eq!(gone[0]["name"], json!("token"));
    }
}
