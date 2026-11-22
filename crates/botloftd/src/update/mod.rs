//! `botloftd update` (spec 42.11): on a Linux server set up by install.sh,
//! the newest release's daemon takes this one's place, checked against the
//! updater's signature, at a moment no agent is working, and the service
//! starts it again. install.sh runs it every hour with a timer.

mod install;
mod signature;
mod version;

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, anyhow, bail};
use botloft_core::protocol::method;
use serde::Deserialize;
use serde_json::json;

use self::install::{install, restart};
pub use self::signature::{UPDATE_KEY, signed};
pub use self::version::Version;
use crate::config::Config;
use crate::local::Owner;

const RELEASES: &str = "https://github.com/httpsphl/botloft/releases";
/// How long a found update waits for the agents to be idle before it goes
/// in anyway.
const WAIT_FOR_IDLE: Duration = Duration::from_secs(24 * 60 * 60);
/// Beside the settings: the version found and when, while it waits.
const PENDING: &str = "update-pending";

/// What the owner asked for.
pub struct Options {
    /// Only say whether there is a newer one.
    pub check: bool,
    /// Do not wait for the agents to be idle.
    pub now: bool,
}

#[derive(Deserialize)]
struct Latest {
    version: String,
}

/// `botloftd update`: puts the newest release's daemon in place of this one
/// and restarts the service.
pub fn update(home: &Path, options: &Options) -> anyhow::Result<()> {
    if cfg!(not(target_os = "linux")) {
        bail!("on this computer, the Botloft app updates Botloft");
    }
    if std::env::var_os("BOTLOFT_IN_CONTAINER").is_some() {
        bail!(
            "in a container, update by pulling the new image: docker pull ghcr.io/httpsphl/botloft"
        );
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let latest = runtime.block_on(latest_version())?;
    let running = Version::running();
    let pending = home.join(PENDING);
    if latest <= running {
        let _ = std::fs::remove_file(&pending);
        println!("Botloft {running} is the newest.");
        return Ok(());
    }
    if options.check {
        println!(
            "Botloft {latest} is out; this one is {running}. Update with: sudo botloftd update"
        );
        return Ok(());
    }
    if !crate::platform::running_as_root() {
        bail!("run it as root: sudo botloftd update");
    }
    if !options.now && runtime.block_on(wait_for_idle(home, &pending, latest))? {
        return Ok(());
    }
    let arch = arch()?;
    let archive = format!("{RELEASES}/download/v{latest}/botloftd-linux-{arch}.tar.gz");
    let bytes = runtime.block_on(download(&archive))?;
    let sig = runtime.block_on(download(&format!("{archive}.sig")))?;
    signed(UPDATE_KEY, &bytes, &String::from_utf8_lossy(&sig)).map_err(|err| anyhow!(err))?;
    let current = std::env::current_exe().context("cannot find this program")?;
    install(&bytes, &current)?;
    let _ = std::fs::remove_file(&pending);
    restart();
    println!("Botloft {latest} is in place of {running}.");
    Ok(())
}

async fn latest_version() -> anyhow::Result<Version> {
    let body = download(&format!("{RELEASES}/latest/download/latest.json")).await?;
    let latest: Latest =
        serde_json::from_slice(&body).context("the releases answered something unexpected")?;
    Version::parse(&latest.version).context("the release has no version")
}

async fn download(url: &str) -> anyhow::Result<Vec<u8>> {
    let response = reqwest::get(url)
        .await
        .and_then(reqwest::Response::error_for_status)
        .with_context(|| format!("could not download {url}"))?;
    Ok(response.bytes().await?.to_vec())
}

/// The processor's name in the release's file names.
fn arch() -> anyhow::Result<&'static str> {
    match std::env::consts::ARCH {
        arch @ ("x86_64" | "aarch64") => Ok(arch),
        other => bail!("no Botloft build for {other}"),
    }
}

/// True when the update waits: agents are working, and it was found less
/// than [`WAIT_FOR_IDLE`] ago. A daemon that does not answer has nothing to
/// interrupt.
async fn wait_for_idle(home: &Path, pending: &Path, latest: Version) -> anyhow::Result<bool> {
    let config = Config::load(&home.join("config.toml"), false)?;
    let Ok(token) = crate::secrets::read_owner_token(&home.join("secrets")) else {
        return Ok(false);
    };
    let Ok(busy) = busy_agents(config.port, &token).await else {
        return Ok(false);
    };
    if busy == 0 {
        return Ok(false);
    }
    let now = SystemTime::now();
    match waited(pending, latest, now) {
        Some(waited) if waited >= WAIT_FOR_IDLE => Ok(false),
        Some(_) => {
            println!("{busy} agents are working; Botloft {latest} goes in once they are done.");
            Ok(true)
        }
        None => {
            let since = now
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            std::fs::write(pending, format!("{latest} {since}\n"))
                .with_context(|| format!("cannot write {}", pending.display()))?;
            println!("{busy} agents are working; Botloft {latest} goes in once they are done.");
            Ok(true)
        }
    }
}

/// How many agents are in the middle of a turn, as the daemon says.
pub async fn busy_agents(port: u16, owner_token: &str) -> anyhow::Result<usize> {
    let mut owner = Owner::connect(port, owner_token, "botloftd update").await?;
    let bots = owner.call(method::BOTS_LIST, json!({})).await?;
    owner.close().await;
    let busy = bots
        .as_array()
        .map(|bots| bots.iter().filter(|bot| bot["state"] == "busy").count())
        .unwrap_or(0);
    Ok(busy)
}

/// How long `latest` has waited, from the file a waiting update leaves;
/// `None` for no file or another version.
fn waited(pending: &Path, latest: Version, now: SystemTime) -> Option<Duration> {
    let text = std::fs::read_to_string(pending).ok()?;
    let (found, since) = text.trim().split_once(' ')?;
    if Version::parse(found)? != latest {
        return None;
    }
    let since = UNIX_EPOCH + Duration::from_millis(since.parse().ok()?);
    Some(now.duration_since(since).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_found_update_waits_a_day_for_idle_agents() {
        let dir = tempfile::tempdir().expect("dir");
        let pending = dir.path().join(PENDING);
        let latest = Version::parse("0.23.0").expect("version");
        let found = UNIX_EPOCH + Duration::from_secs(1_800_000_000);
        assert_eq!(waited(&pending, latest, found), None, "nothing found yet");
        let at = found.duration_since(UNIX_EPOCH).expect("time").as_millis();
        std::fs::write(&pending, format!("0.23.0 {at}\n")).expect("write");
        let hour = Duration::from_secs(3600);
        assert_eq!(waited(&pending, latest, found + hour), Some(hour));
        let newer = Version::parse("0.24.0").expect("version");
        assert_eq!(
            waited(&pending, newer, found + hour),
            None,
            "another version"
        );
    }
}
