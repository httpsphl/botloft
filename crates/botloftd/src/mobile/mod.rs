//! The phone (spec 28): connecting one, and the relay that carries sealed
//! approvals and questions to it. Nothing opens until a phone is connected
//! or being connected, and nothing of what passes goes in the log.

mod bridge;
mod cards;
mod chats;
mod kdf;
mod pair;
mod phones;
mod relay;
pub mod seal;
mod talk;

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use botloft_core::protocol::{
    MobileJoined, MobilePairing, MobilePhone, MobileRelay, MobileStatus, SEALED_MAX, ToPhone,
};
use tokio::sync::{Notify, mpsc};
use tracing::warn;

pub use self::pair::Confirm;
use self::phones::{Phone, Phones};
pub use self::relay::run;
use self::seal::{Dir, Keypair, Keys};
use self::talk::Talk;
use crate::cloud::{CloudError, Credentials, Server};
use crate::state::{Daemon, Event};

/// How soon the relay tries again after losing the server, and how long it
/// waits at most between tries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retry {
    pub min: Duration,
    pub max: Duration,
}

impl Default for Retry {
    fn default() -> Self {
        Self {
            min: Duration::from_secs(1),
            max: Duration::from_secs(60),
        }
    }
}

/// A QR code on screen, with the secret only the phone that scans it shares.
pub(crate) struct Pairing {
    pub(crate) id: String,
    pub(crate) secret: [u8; 32],
    pub(crate) keys: Keypair,
    pub(crate) expires_at: i64,
    pub(crate) joined: Option<Joined>,
}

/// A phone that opened the code and waits for the owner.
pub(crate) struct Joined {
    pub(crate) name: String,
    pub(crate) phone_pub: Vec<u8>,
    pub(crate) code: String,
    /// The owner accepted: the keys, until the phone collects its token and
    /// the server says which device it is.
    pub(crate) accepted: Option<Keys>,
}

#[derive(Default)]
struct State {
    relay: Option<MobileRelay>,
    online: HashSet<String>,
    pairing: Option<Pairing>,
    /// When each phone answered lately, for the limit.
    answers: HashMap<String, VecDeque<i64>>,
    /// The connection's way out, while there is one.
    live: Option<mpsc::UnboundedSender<String>>,
}

pub struct Mobile {
    phones: Phones,
    state: Mutex<State>,
    talk: Mutex<Talk>,
    wake: Notify,
    retry: Retry,
}

/// The server and the token of this sign-in.
pub(crate) fn identity(daemon: &Daemon) -> Result<(Server, Credentials), CloudError> {
    let server = daemon.cloud.server()?;
    let credentials = daemon
        .cloud
        .credentials(&daemon.paths.secrets())
        .ok_or_else(CloudError::not_signed_in)?;
    Ok((server, credentials))
}

impl Mobile {
    pub fn new(secrets_dir: &Path, retry: Retry) -> Self {
        Self {
            phones: Phones::open(secrets_dir),
            state: Mutex::new(State::default()),
            talk: Mutex::new(Talk::default()),
            wake: Notify::new(),
            retry,
        }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Asks the relay to look at what is wanted again.
    pub(crate) fn wake(&self) {
        self.wake.notify_one();
    }

    /// The phones of this sign-in; none when signed out.
    pub(crate) fn phones(&self, daemon: &Daemon) -> Vec<Phone> {
        match daemon.cloud.credentials(&daemon.paths.secrets()) {
            Some(credentials) => self.phones.list(&credentials.url, &credentials.device),
            None => Vec::new(),
        }
    }

    /// Whether the relay should be open: signed in, and a phone connected or
    /// being connected.
    pub(crate) fn wanted(&self, daemon: &Daemon) -> bool {
        daemon.cloud.credentials(&daemon.paths.secrets()).is_some()
            && (!self.phones(daemon).is_empty() || self.lock().pairing.is_some())
    }

    pub fn status(&self, daemon: &Daemon) -> MobileStatus {
        let now = daemon.clock.now_ms();
        let mut state = self.lock();
        if state
            .pairing
            .as_ref()
            .is_some_and(|pairing| pairing.expires_at <= now)
        {
            state.pairing = None;
        }
        let phones = self
            .phones(daemon)
            .into_iter()
            .map(|phone| MobilePhone {
                online: state.online.contains(&phone.id),
                id: phone.id,
                name: phone.name,
                paired_at: phone.paired_at,
                last_seen_at: phone.last_seen_at,
            })
            .collect();
        let pending = state.pairing.as_ref().map(|pairing| MobilePairing {
            pair_id: pairing.id.clone(),
            expires_at: pairing.expires_at,
            joined: pairing.joined.as_ref().map(|joined| MobileJoined {
                name: joined.name.clone(),
                code: joined.code.clone(),
            }),
        });
        MobileStatus {
            relay: state.relay.unwrap_or(MobileRelay::Off),
            phones,
            pending,
        }
    }

    /// Tells the apps how things stand now.
    pub(crate) fn changed(&self, daemon: &Daemon) {
        daemon.emit(Event::MobileChanged(self.status(daemon)));
    }

    pub(crate) fn set_relay(&self, daemon: &Daemon, relay: MobileRelay) {
        let changed = {
            let mut state = self.lock();
            let changed = state.relay.unwrap_or(MobileRelay::Off) != relay;
            state.relay = Some(relay);
            if relay != MobileRelay::Connected {
                state.live = None;
                state.online.clear();
            }
            changed
        };
        if relay != MobileRelay::Connected {
            self.leave_all();
        }
        if changed {
            self.changed(daemon);
        }
    }

    pub(crate) fn set_live(&self, live: Option<mpsc::UnboundedSender<String>>) {
        self.lock().live = live;
    }

    /// Hands a frame to the relay connection, if there is one.
    pub(crate) fn send_frame(&self, frame: &serde_json::Value) -> bool {
        self.lock()
            .live
            .as_ref()
            .is_some_and(|live| live.send(frame.to_string()).is_ok())
    }

    pub(crate) fn set_online(&self, online: HashSet<String>) {
        self.lock().online = online;
    }

    /// A phone came or went. False if it is not one of ours.
    pub(crate) fn presence(&self, daemon: &Daemon, device: &str, online: bool) -> bool {
        if !self.phones(daemon).iter().any(|phone| phone.id == device) {
            return false;
        }
        {
            let mut state = self.lock();
            if online {
                state.online.insert(device.to_owned());
            } else {
                state.online.remove(device);
            }
        }
        if !online {
            self.leave(device);
        }
        self.changed(daemon);
        true
    }

    pub(crate) fn online_phones(&self, daemon: &Daemon) -> Vec<Phone> {
        let state = self.lock();
        self.phones(daemon)
            .into_iter()
            .filter(|phone| state.online.contains(&phone.id))
            .collect()
    }

    /// Seals `message` for the phone and hands it to the relay. The counter
    /// is saved before the message goes. Nothing happens if the relay is not
    /// connected: the phone asks for what it missed when it opens.
    pub(crate) fn send_to(&self, daemon: &Daemon, phone: &str, message: &ToPhone) {
        let Some((_, credentials)) = identity(daemon).ok() else {
            return;
        };
        let Some(known) = self
            .phones
            .get(&credentials.url, &credentials.device, phone)
        else {
            return;
        };
        let Some(keys) = known.keys() else {
            return;
        };
        let Ok(plain) = serde_json::to_vec(message) else {
            return;
        };
        if plain.len() > SEALED_MAX {
            warn!("a message for a phone is too big and was not sent");
            return;
        }
        let seq = known.sent + 1;
        let saved = self
            .phones
            .update(&credentials.url, &credentials.device, phone, |phone| {
                phone.sent = seq;
            });
        if !matches!(saved, Ok(true)) {
            warn!("could not save a phone's counter; nothing was sent");
            return;
        }
        match seal::seal(&keys.c2p, Dir::ComputerToPhone, phone, seq, &plain) {
            Ok(body) => {
                self.send_frame(&serde_json::json!({
                    "t": "msg", "to": phone, "seq": seq, "body": body,
                }));
            }
            Err(_) => warn!("a message for a phone did not seal"),
        }
    }

    /// The computer left the account: every phone goes with it.
    pub fn forget(&self, daemon: &Daemon) {
        self.phones.clear();
        {
            let mut state = self.lock();
            state.pairing = None;
            state.online.clear();
            state.answers.clear();
        }
        self.leave_all();
        self.wake();
        self.changed(daemon);
    }

    /// Whether the phone may answer now: at most 30 a minute.
    pub(crate) fn may_answer(&self, daemon: &Daemon, phone: &str) -> bool {
        self.within_limit(daemon, phone.to_owned())
    }

    /// Whether the phone may write to a bot now: at most 30 a minute, apart
    /// from the answers (spec 28.12).
    pub(crate) fn may_send(&self, daemon: &Daemon, phone: &str) -> bool {
        self.within_limit(daemon, format!("send:{phone}"))
    }

    fn within_limit(&self, daemon: &Daemon, key: String) -> bool {
        const PER_MINUTE: usize = 30;
        let now = daemon.clock.now_ms();
        let mut state = self.lock();
        let recent = state.answers.entry(key).or_default();
        while recent.front().is_some_and(|at| *at <= now - 60_000) {
            recent.pop_front();
        }
        if recent.len() >= PER_MINUTE {
            return false;
        }
        recent.push_back(now);
        true
    }
}
