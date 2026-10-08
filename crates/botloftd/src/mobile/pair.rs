//! Connecting a phone (spec 28.3): the QR code, the phone that joins, the
//! owner who compares the code, and the phone that collects its token.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use botloft_core::protocol::{MobilePairRequest, MobilePairStarted};
use tracing::warn;

use super::kdf::same;
use super::phones::Phone;
use super::seal::{self, Keypair};
use super::{Joined, Mobile, Pairing, identity};
use crate::cloud::{CloudError, Seen};
use crate::state::{Daemon, Event};

/// Why a confirmation could not be taken.
#[derive(Debug, PartialEq, Eq)]
pub enum Confirm {
    /// There is no such code, or no phone has joined it.
    NothingToConfirm,
    Cloud(CloudError),
}

impl From<CloudError> for Confirm {
    fn from(err: CloudError) -> Self {
        Self::Cloud(err)
    }
}

fn random<const N: usize>() -> Result<[u8; N], CloudError> {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes)
        .map_err(|_| CloudError::other("the system has no random bytes".to_owned()))?;
    Ok(bytes)
}

impl Mobile {
    /// `mobile.pair_start`: a code for a phone to scan. Another code on
    /// screen is dropped.
    pub async fn pair_start(&self, daemon: &Daemon) -> Result<MobilePairStarted, CloudError> {
        let (server, credentials) = identity(daemon)?;
        let id = B64.encode(random::<16>()?);
        let secret = random::<32>()?;
        let keys = Keypair::generate()
            .map_err(|_| CloudError::other("the system has no random bytes".to_owned()))?;
        let daemon_pub = B64.encode(keys.public());
        let old = self.lock().pairing.take();
        if let Some(old) = old {
            let _ = server.cancel_pairing(&credentials.token, &old.id).await;
        }
        let expires_in = server.open_pairing(&credentials.token, &id).await?;
        let expires_at = daemon.clock.now_ms() + i64::from(expires_in) * 1000;
        // The part after `#` never reaches the server. The computer's public
        // key is there so the phone can show the code before it is accepted.
        let url = format!(
            "{}#p={id}&s={}&d={}",
            server.phone_page(),
            B64.encode(secret),
            daemon_pub
        );
        self.lock().pairing = Some(Pairing {
            id: id.clone(),
            secret,
            keys,
            expires_at,
            joined: None,
        });
        self.wake();
        self.changed(daemon);
        Ok(MobilePairStarted {
            pair_id: id,
            url,
            expires_in,
        })
    }

    /// `mobile.pair_cancel`.
    pub async fn pair_cancel(&self, daemon: &Daemon, id: &str) -> Result<(), CloudError> {
        let ours = {
            let mut state = self.lock();
            if state
                .pairing
                .as_ref()
                .is_some_and(|pairing| pairing.id == id)
            {
                state.pairing = None;
                true
            } else {
                false
            }
        };
        if ours {
            if let Ok((server, credentials)) = identity(daemon) {
                let _ = server.cancel_pairing(&credentials.token, id).await;
            }
            self.wake();
            self.changed(daemon);
        }
        Ok(())
    }

    /// Looks whether a phone opened the code, and if its proof is right,
    /// shows the owner its name and the code to compare.
    pub(crate) async fn check_joined(&self, daemon: &Daemon) {
        let Some((id, secret, daemon_pub)) = ({
            let state = self.lock();
            state
                .pairing
                .as_ref()
                .filter(|pairing| pairing.joined.is_none())
                .map(|pairing| {
                    (
                        pairing.id.clone(),
                        pairing.secret,
                        pairing.keys.public().to_vec(),
                    )
                })
        }) else {
            return;
        };
        let Ok((server, credentials)) = identity(daemon) else {
            return;
        };
        let seen = server.pairing(&credentials.token, &id).await;
        let (name, phone_pub, proof) = match seen {
            Ok(Seen::Joined {
                name,
                phone_pub,
                proof,
            }) => (name, phone_pub, proof),
            Ok(Seen::Gone) => {
                let mut state = self.lock();
                if state.pairing.as_ref().is_some_and(|p| p.id == id) {
                    state.pairing = None;
                    drop(state);
                    self.changed(daemon);
                }
                return;
            }
            Ok(Seen::Waiting) | Err(_) => return,
        };
        let phone_pub = B64.decode(&phone_pub).unwrap_or_default();
        let proof = B64.decode(&proof).unwrap_or_default();
        let right = same(&seal::join_proof(&secret, &id, &phone_pub), &proof);
        if !right || phone_pub.len() != 65 {
            warn!("a phone joined the code with a wrong proof and was refused");
            let _ = server.cancel_pairing(&credentials.token, &id).await;
            self.lock().pairing = None;
            self.changed(daemon);
            return;
        }
        let code = seal::code(&secret, &daemon_pub, &phone_pub);
        {
            let mut state = self.lock();
            let Some(pairing) = state.pairing.as_mut().filter(|pairing| pairing.id == id) else {
                return;
            };
            pairing.joined = Some(Joined {
                name: name.clone(),
                phone_pub,
                code: code.clone(),
                accepted: None,
            });
        }
        self.changed(daemon);
        daemon.emit(Event::MobilePairRequest(MobilePairRequest {
            pair_id: id,
            name,
            code,
        }));
    }

    /// `mobile.pair_confirm`: the owner compared the codes.
    pub async fn pair_confirm(
        &self,
        daemon: &Daemon,
        id: &str,
        accept: bool,
    ) -> Result<(), Confirm> {
        let (server, credentials) = identity(daemon)?;
        let work = {
            let state = self.lock();
            state
                .pairing
                .as_ref()
                .filter(|pairing| pairing.id == id)
                .and_then(|pairing| {
                    let joined = pairing.joined.as_ref().filter(|j| j.accepted.is_none())?;
                    Some((
                        pairing.secret,
                        pairing.keys.public().to_vec(),
                        joined.phone_pub.clone(),
                        seal::derive(&pairing.keys, &joined.phone_pub, &pairing.secret).ok()?,
                    ))
                })
        };
        let Some((secret, daemon_pub, phone_pub, keys)) = work else {
            return Err(Confirm::NothingToConfirm);
        };
        if !accept {
            server.cancel_pairing(&credentials.token, id).await?;
            self.lock().pairing = None;
            self.wake();
            self.changed(daemon);
            return Ok(());
        }
        let proof = seal::accept_proof(&secret, id, &daemon_pub, &phone_pub);
        server
            .accept_pairing(
                &credentials.token,
                id,
                &B64.encode(&daemon_pub),
                &B64.encode(proof),
            )
            .await?;
        if let Some(joined) = self
            .lock()
            .pairing
            .as_mut()
            .and_then(|pairing| pairing.joined.as_mut())
        {
            joined.accepted = Some(keys);
        }
        Ok(())
    }

    /// The server says the phone collected its token and which device it is:
    /// from now on the keys are kept for that device.
    pub(crate) fn paired(&self, daemon: &Daemon, pairing_id: &str, device: &str) {
        let Ok((_, credentials)) = identity(daemon) else {
            return;
        };
        let taken = {
            let mut state = self.lock();
            let ours = state
                .pairing
                .as_ref()
                .is_some_and(|pairing| pairing.id == pairing_id);
            if ours { state.pairing.take() } else { None }
        };
        let Some(Pairing {
            joined:
                Some(Joined {
                    name,
                    accepted: Some(keys),
                    ..
                }),
            ..
        }) = taken
        else {
            return;
        };
        let phone = Phone::new(device.to_owned(), name, daemon.clock.now_ms(), &keys);
        if let Err(err) = self
            .phones
            .add(&credentials.url, &credentials.device, phone)
        {
            warn!("could not keep the new phone's keys: {err}");
            return;
        }
        self.changed(daemon);
    }

    /// Whether a code was accepted and waits for its phone to collect the
    /// token: a device the server lists then is not a stranger.
    pub(crate) fn awaiting_phone(&self) -> bool {
        self.lock()
            .pairing
            .as_ref()
            .and_then(|pairing| pairing.joined.as_ref())
            .is_some_and(|joined| joined.accepted.is_some())
    }

    /// `mobile.revoke`: the server is told if it can be; the keys go either
    /// way, and a phone the server still lists is taken off at the next
    /// connection.
    pub async fn revoke(&self, daemon: &Daemon, phone: &str) -> Result<bool, CloudError> {
        let (server, credentials) = identity(daemon)?;
        let removed = self
            .phones
            .remove(&credentials.url, &credentials.device, phone)
            .unwrap_or(false);
        let _ = server.remove_device(&credentials.token, phone).await;
        self.lock().online.remove(phone);
        self.wake();
        self.changed(daemon);
        Ok(removed)
    }

    /// The phone left from its side: the server tells us, and the keys go.
    pub(crate) fn revoked(&self, daemon: &Daemon, phone: &str) {
        if let Ok((_, credentials)) = identity(daemon) {
            let _ = self
                .phones
                .remove(&credentials.url, &credentials.device, phone);
        }
        self.lock().online.remove(phone);
        self.wake();
        self.changed(daemon);
    }
}
