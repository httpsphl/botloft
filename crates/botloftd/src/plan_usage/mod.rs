//! The plan's usage read straight from the owner's Claude account (spec
//! 8.13), the way Claude Code's `/usage` reads it, with the login Claude
//! Code keeps on this computer. Only while someone looks: reading
//! `system.status` asks for a new reading once the last is a minute old.
//! The login is only read, never renewed or written, never logged, and only
//! sent to Anthropic.

mod parse;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use botloft_core::protocol::{AccountUsage, UsageWindow};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use tokio::sync::Notify;
use tracing::debug;

use crate::state::Daemon;

/// What Claude Code's `/usage` asks (spec 19).
const URL: &str = "https://api.anthropic.com/api/oauth/usage";
const BETA: &str = "oauth-2025-04-20";
const TIMEOUT: Duration = Duration::from_secs(10);
/// A reading this fresh is not asked again.
const FRESH_MS: i64 = 60_000;
/// The wait after a reading that did not work.
const RETRY_MS: i64 = 5 * 60_000;

/// When the plan may be read again, and the nudge that asks for it.
pub struct PlanReader {
    wanted: Notify,
    next: Mutex<i64>,
    /// Where it asks; tests point it at a server of their own.
    url: Mutex<String>,
    /// Claude Code's folder; `None` finds it in the owner's environment.
    root: Mutex<Option<PathBuf>>,
}

impl Default for PlanReader {
    fn default() -> Self {
        Self {
            wanted: Notify::new(),
            next: Mutex::new(0),
            url: Mutex::new(URL.to_owned()),
            root: Mutex::new(None),
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl PlanReader {
    /// Someone looks at the plan: read it again if it is time.
    pub fn wanted(&self) {
        self.wanted.notify_one();
    }

    /// Tests: ask `url`, with the login in `root`.
    pub fn point_at(&self, url: &str, root: &Path) {
        *lock(&self.url) = url.to_owned();
        *lock(&self.root) = Some(root.to_owned());
    }
}

/// Why a reading did not work; said only in debug logs.
#[derive(Debug)]
enum Missed {
    NoFolder,
    NoLogin,
    Expired,
    Refused,
    Network,
    Unreadable,
}

/// Reads the plan each time someone looks and it is time to.
pub async fn run(daemon: Arc<Daemon>) {
    loop {
        daemon.plan_reader.wanted.notified().await;
        if daemon.clock.now_ms() < *lock(&daemon.plan_reader.next) {
            continue;
        }
        let read = read(&daemon).await;
        let now = daemon.clock.now_ms();
        let wait = match read {
            Ok(windows) => {
                daemon.set_account_windows(windows, now);
                FRESH_MS
            }
            Err(missed) => {
                debug!("the plan's usage stays as the agents saw it: {missed:?}");
                RETRY_MS
            }
        };
        *lock(&daemon.plan_reader.next) = now + wait;
    }
}

async fn read(daemon: &Daemon) -> Result<Vec<UsageWindow>, Missed> {
    let root = lock(&daemon.plan_reader.root).clone();
    let root = match root {
        Some(root) => root,
        None => crate::imports::claude_code::root(
            &crate::service::import_sources::environment(),
            dirs::home_dir(),
        )
        .ok_or(Missed::NoFolder)?,
    };
    let text =
        std::fs::read_to_string(root.join(".credentials.json")).map_err(|_| Missed::NoLogin)?;
    let token = parse::token(&text, daemon.clock.now_ms())?;
    let url = lock(&daemon.plan_reader.url).clone();
    let client = Client::builder()
        .timeout(TIMEOUT)
        .user_agent(concat!("botloftd/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| Missed::Network)?;
    let response = client
        .get(url)
        .bearer_auth(token)
        .header("anthropic-beta", BETA)
        .send()
        .await
        .map_err(|_| Missed::Network)?;
    match response.status() {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => return Err(Missed::Refused),
        status if !status.is_success() => return Err(Missed::Network),
        _ => {}
    }
    let body: Value = response.json().await.map_err(|_| Missed::Unreadable)?;
    parse::windows(&body).ok_or(Missed::Unreadable)
}

/// What the agents saw in a `rate_limit_event` (8.1): its status always,
/// and its windows over the ones of the same name, keeping the account's
/// others (a model's week) when the windows came from the account.
pub(crate) fn merge(current: &mut Option<AccountUsage>, seen: AccountUsage) {
    match current {
        Some(was) if was.from_account => {
            for window in seen.windows {
                let same = was
                    .windows
                    .iter_mut()
                    .find(|kept| kept.name == window.name && kept.model.is_none());
                match same {
                    Some(kept) => *kept = window,
                    None => was.windows.push(window),
                }
            }
            was.status = seen.status;
            was.resets_at = seen.resets_at;
            was.extra_usage = seen.extra_usage;
            was.observed_at = seen.observed_at;
        }
        _ => *current = Some(seen),
    }
}

/// The windows the account gave, read at `now`; the status stays as the
/// agents last saw it.
pub(crate) fn from_account(
    current: &mut Option<AccountUsage>,
    windows: Vec<UsageWindow>,
    now: i64,
) {
    match current {
        Some(usage) => {
            usage.windows = windows;
            usage.observed_at = now;
            usage.from_account = true;
        }
        None => {
            *current = Some(AccountUsage {
                status: "allowed".to_owned(),
                resets_at: None,
                windows,
                observed_at: now,
                extra_usage: false,
                from_account: true,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(name: &str, utilization: f64, model: Option<&str>) -> UsageWindow {
        UsageWindow {
            name: name.to_owned(),
            utilization,
            resets_at: None,
            model: model.map(str::to_owned),
        }
    }

    fn seen(windows: Vec<UsageWindow>) -> AccountUsage {
        AccountUsage {
            status: "allowed_warning".to_owned(),
            resets_at: Some(9),
            windows,
            observed_at: 20,
            extra_usage: false,
            from_account: false,
        }
    }

    #[test]
    fn what_the_agents_see_updates_the_account_reading_without_dropping_its_windows() {
        let mut usage = None;
        merge(&mut usage, seen(vec![window("seven_day", 0.5, None)]));
        assert!(!usage.as_ref().expect("usage").from_account);

        let account = vec![
            window("seven_day", 0.79, None),
            window("seven_day", 0.3, Some("Fable")),
        ];
        from_account(&mut usage, account, 30);
        let read = usage.clone().expect("usage");
        assert!(read.from_account);
        assert_eq!(read.status, "allowed_warning", "the status stays");
        assert_eq!(read.windows.len(), 2);

        merge(&mut usage, seen(vec![window("seven_day", 0.8, None)]));
        let merged = usage.expect("usage");
        assert!(merged.from_account);
        assert_eq!(merged.windows.len(), 2);
        assert!((merged.windows[0].utilization - 0.8).abs() < 1e-9);
        assert_eq!(merged.windows[1].model.as_deref(), Some("Fable"));
        assert_eq!(merged.observed_at, 20);
    }
}
