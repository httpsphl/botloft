//! `GET /download/<system>` (spec 39.3): straight to the installer of the
//! newest release, so the site never names an old version. The server asks
//! the releases page which version is the newest at most once every ten
//! minutes, and says nothing about who clicked. Without an answer, the
//! visitor lands on the releases page and picks the file there.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path as UrlPath, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use tokio::sync::Mutex;

use crate::app::AppState;

/// How long a newest version is trusted, and how long a failed look waits.
const FRESH_MS: i64 = 10 * 60 * 1000;
const RETRY_MS: i64 = 60 * 1000;

/// The installer of each system, after `Botloft_<version>_` (the names the
/// release workflow gives them, spec 15.5).
const INSTALLERS: [(&str, &str); 4] = [
    ("windows", "x64-setup.exe"),
    ("macos", "aarch64.dmg"),
    ("linux-deb", "amd64.deb"),
    ("linux-appimage", "amd64.AppImage"),
];

/// What the server knows of the newest release.
#[derive(Clone)]
pub struct Releases(Arc<Inner>);

struct Inner {
    http: reqwest::Client,
    newest: Mutex<Option<Known>>,
}

struct Known {
    tag: Option<String>,
    until: i64,
}

impl Releases {
    pub fn new() -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .user_agent("botloft-cloud")
            .build()?;
        Ok(Self(Arc::new(Inner {
            http,
            newest: Mutex::new(None),
        })))
    }

    /// The newest release's tag, such as `v0.19.0`, from `base` (a releases
    /// page) or from what was learned less than ten minutes ago.
    async fn newest(&self, base: &str, now: i64) -> Option<String> {
        let mut known = self.0.newest.lock().await;
        if let Some(known) = known.as_ref().filter(|known| now < known.until) {
            return known.tag.clone();
        }
        let tag = self.ask(base).await;
        let wait = if tag.is_some() { FRESH_MS } else { RETRY_MS };
        *known = Some(Known {
            tag: tag.clone(),
            until: now + wait,
        });
        tag
    }

    /// `<base>/latest` answers with a redirect to the newest release's page.
    async fn ask(&self, base: &str) -> Option<String> {
        let reply = match self.0.http.get(format!("{base}/latest")).send().await {
            Ok(reply) => reply,
            Err(err) => {
                tracing::warn!("the newest release could not be learned: {err}");
                return None;
            }
        };
        let location = reply
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok());
        let tag = location.and_then(tag_of);
        if tag.is_none() {
            tracing::warn!(
                "the newest release could not be learned: {} with no release in it",
                reply.status()
            );
        }
        tag
    }
}

/// The tag at the end of a release page's address, if it is a plain
/// version (`v0.19.0`, `v1.0.0-rc.1`): it goes into a link.
fn tag_of(location: &str) -> Option<String> {
    let (_, tag) = location.rsplit_once("/tag/")?;
    let version = tag.strip_prefix('v')?;
    let plain = version.len() <= 40
        && version.starts_with(|first: char| first.is_ascii_digit())
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    plain.then(|| tag.to_owned())
}

/// `GET /download/<system>`: `windows`, `macos`, `linux-deb` or
/// `linux-appimage`.
pub async fn installer(
    State(state): State<AppState>,
    UrlPath(system): UrlPath<String>,
) -> Response {
    let Some(site) = state.config.site.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some((_, suffix)) = INSTALLERS.iter().find(|(name, _)| *name == system) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let base = site.releases.trim_end_matches('/');
    let target = match state.releases.newest(base, state.clock.now()).await {
        Some(tag) => {
            let version = tag.trim_start_matches('v');
            format!("{base}/download/{tag}/Botloft_{version}_{suffix}")
        }
        None => format!("{base}/latest"),
    };
    (
        StatusCode::FOUND,
        [
            (header::LOCATION, target),
            (header::CACHE_CONTROL, "no-store".to_owned()),
        ],
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::tag_of;

    #[test]
    fn only_a_plain_version_is_a_tag() {
        let page = "https://github.com/httpsphl/botloft/releases/tag/";
        assert_eq!(tag_of(&format!("{page}v0.19.0")), Some("v0.19.0".into()));
        assert_eq!(
            tag_of(&format!("{page}v1.0.0-rc.1")),
            Some("v1.0.0-rc.1".into())
        );
        assert_eq!(tag_of("https://github.com/httpsphl/botloft/releases"), None);
        for bad in ["0.19.0", "v", "vx", "v1.0/../../x", "v1.0?x=1", "v1.0%2f"] {
            assert_eq!(tag_of(&format!("{page}{bad}")), None, "{bad}");
        }
    }
}
