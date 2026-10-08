//! Web Push (spec 28.8): when the computer says something waits, each phone
//! connected with it gets a push with **no content**. The phone's own page
//! writes the notice. The server signs its call with a key of its own (VAPID,
//! RFC 8292) and calls only the push services it was told to, never an
//! address a phone made up.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use reqwest::Url;
use rusqlite::{Connection, params};

use crate::app::AppState;
use crate::config::{Config, VAPID_KEY_VAR, secret};

/// The push services of the browsers, which a phone may name by default.
const DEFAULT_HOSTS: &[&str] = &[
    "fcm.googleapis.com",
    "push.services.mozilla.com",
    "notify.windows.com",
    "push.apple.com",
];

/// How long apart two notices to the same computer's phones go; the ones in
/// between merge into a single notice at the end of the wait.
pub const WAKE_GAP: Duration = Duration::from_secs(30);

/// A new key for the server: the private half as the file holds it, and the
/// public half the phones need (`applicationServerKey`), both base64url.
pub fn new_key() -> anyhow::Result<(String, String)> {
    loop {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|err| anyhow::anyhow!("no random bytes: {err}"))?;
        // Not every 32 bytes are a key; try again.
        if let Ok(key) = SigningKey::from_slice(&bytes) {
            return Ok((B64.encode(bytes), public_of(&key)));
        }
    }
}

fn public_of(key: &SigningKey) -> String {
    B64.encode(key.verifying_key().to_sec1_point(false).as_bytes())
}

struct Inner {
    key: SigningKey,
    public: String,
    subject: String,
    allowed: Vec<String>,
    gap: Duration,
    http: reqwest::Client,
    /// Per computer: when its phones were last told, and whether a notice is
    /// already set for the end of the wait.
    told: Mutex<HashMap<String, (Instant, bool)>>,
}

/// What sends the notices; off when `[push]` is not configured.
#[derive(Clone, Default)]
pub struct Pusher(Option<Arc<Inner>>);

impl Pusher {
    pub fn off() -> Self {
        Self(None)
    }

    pub fn from_config(config: &Config) -> anyhow::Result<Self> {
        let Some(push) = &config.push else {
            return Ok(Self::off());
        };
        let text = secret(push.vapid_key_file.as_deref(), VAPID_KEY_VAR, "VAPID key")?;
        Self::new(&text, &push.subject, &push.allow_hosts, WAKE_GAP)
    }

    /// `key` is the private half in base64url, as `new_key` made it.
    pub fn new(key: &str, subject: &str, allow: &[String], gap: Duration) -> anyhow::Result<Self> {
        let bytes = B64
            .decode(key.trim())
            .map_err(|_| anyhow::anyhow!("the VAPID key is not base64url"))?;
        let key = SigningKey::from_slice(&bytes)
            .map_err(|_| anyhow::anyhow!("the VAPID key is not a P-256 key"))?;
        let allowed = if allow.is_empty() {
            DEFAULT_HOSTS
                .iter()
                .map(|host| (*host).to_owned())
                .collect()
        } else {
            allow.iter().map(|host| host.to_ascii_lowercase()).collect()
        };
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("botloft-cloud/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self(Some(Arc::new(Inner {
            public: public_of(&key),
            key,
            subject: subject.to_owned(),
            allowed,
            gap,
            http,
            told: Mutex::default(),
        }))))
    }

    /// The public key a phone subscribes with, if pushes are on.
    pub fn public_key(&self) -> Option<&str> {
        self.0.as_deref().map(|inner| inner.public.as_str())
    }

    /// Whether `endpoint` is a push service the server may call: `https`, no
    /// credentials, a host that is one of the allowed or under one of them.
    pub fn allows(&self, endpoint: &str) -> bool {
        let Some(inner) = &self.0 else {
            return false;
        };
        let Ok(url) = Url::parse(endpoint) else {
            return false;
        };
        let local = matches!(url.host_str(), Some("127.0.0.1" | "localhost"));
        let scheme_ok = url.scheme() == "https" || (url.scheme() == "http" && local);
        let Some(host) = url.host_str().map(str::to_ascii_lowercase) else {
            return false;
        };
        scheme_ok
            && url.username().is_empty()
            && url.password().is_none()
            && inner
                .allowed
                .iter()
                .any(|allowed| host == *allowed || host.ends_with(&format!(".{allowed}")))
    }

    /// Something waits for the phones of `computer`: tell them, now if they
    /// were not told lately, else once when the wait is over.
    pub fn wake(&self, state: &AppState, computer: &str) {
        let Some(inner) = &self.0 else {
            return;
        };
        let wait = {
            let mut told = inner
                .told
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let entry = told.entry(computer.to_owned()).or_insert((
                Instant::now()
                    .checked_sub(inner.gap)
                    .unwrap_or_else(Instant::now),
                false,
            ));
            let since = entry.0.elapsed();
            if since >= inner.gap {
                *entry = (Instant::now(), false);
                Duration::ZERO
            } else if entry.1 {
                return;
            } else {
                entry.1 = true;
                inner.gap - since
            }
        };
        let (inner, state, computer) = (Arc::clone(inner), state.clone(), computer.to_owned());
        tokio::spawn(async move {
            if !wait.is_zero() {
                tokio::time::sleep(wait).await;
                let mut told = inner
                    .told
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                told.insert(computer.clone(), (Instant::now(), false));
            }
            let endpoints = state
                .db
                .run(|conn| endpoints_of(conn, &computer))
                .unwrap_or_default();
            for (device, endpoint) in endpoints {
                let sent = send(&inner, &endpoint, state.clock.now()).await;
                if sent == Outcome::Gone {
                    let _ = state.db.run(|conn| forget(conn, &device));
                }
            }
        });
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    Sent,
    /// The push service says the phone is not reachable there any more.
    Gone,
    Failed,
}

/// The empty push: a signed call with a time to live and nothing in it.
async fn send(inner: &Inner, endpoint: &str, now_ms: i64) -> Outcome {
    let Ok(url) = Url::parse(endpoint) else {
        return Outcome::Failed;
    };
    let audience = url.origin().ascii_serialization();
    let jwt = jwt(inner, &audience, now_ms / 1000);
    let sent = inner
        .http
        .post(url)
        .header(
            "Authorization",
            format!("vapid t={jwt}, k={}", inner.public),
        )
        .header("TTL", "3600")
        .header("Urgency", "high")
        .header("Content-Length", "0")
        .send()
        .await;
    match sent.map(|response| response.status().as_u16()) {
        Ok(200..=202) => Outcome::Sent,
        Ok(404 | 410) => Outcome::Gone,
        Ok(status) => {
            tracing::warn!(status, "a push service refused a notice");
            Outcome::Failed
        }
        Err(_) => {
            tracing::warn!("a push service could not be reached");
            Outcome::Failed
        }
    }
}

/// The signed token of RFC 8292: the push service's origin, twelve hours.
fn jwt(inner: &Inner, audience: &str, now: i64) -> String {
    let claims =
        serde_json::json!({ "aud": audience, "exp": now + 12 * 3600, "sub": inner.subject });
    let head = B64.encode(br#"{"typ":"JWT","alg":"ES256"}"#);
    let body = B64.encode(claims.to_string());
    let signing = format!("{head}.{body}");
    let signature: Signature = inner.key.sign(signing.as_bytes());
    format!("{signing}.{}", B64.encode(signature.to_bytes()))
}

/// `(phone, endpoint)` for every phone connected with `computer` that has one.
fn endpoints_of(conn: &Connection, computer: &str) -> rusqlite::Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT devices.id, push_subscriptions.endpoint FROM devices \
         JOIN push_subscriptions ON push_subscriptions.device = devices.id \
         WHERE devices.peer = ?1 AND devices.kind = 'phone'",
    )?;
    stmt.query_map([computer], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect()
}

pub fn subscribe(
    conn: &Connection,
    device: &str,
    endpoint: &str,
    now: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO push_subscriptions (device, endpoint, created_at) VALUES (?1, ?2, ?3) \
         ON CONFLICT(device) DO UPDATE SET endpoint = excluded.endpoint, created_at = excluded.created_at",
        params![device, endpoint, now],
    )?;
    Ok(())
}

pub fn forget(conn: &Connection, device: &str) -> rusqlite::Result<bool> {
    let removed = conn.execute("DELETE FROM push_subscriptions WHERE device = ?1", [device])?;
    Ok(removed > 0)
}
