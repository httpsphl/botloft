//! Signing in by the link in the e-mail (spec 27.3): ask, then wait in the
//! background until the owner opens it, then keep the token.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use botloft_core::protocol::CloudSignedIn;
use tokio::sync::broadcast;

use super::client::{Polled, Server};
use super::creds::{self, Credentials};
use super::{Cloud, CloudError};
use crate::state::Event;

/// What a sign-in needs from the daemon.
pub struct Signin {
    pub email: String,
    pub locale: String,
    pub secrets: PathBuf,
    pub events: broadcast::Sender<Event>,
}

/// The name of this computer, for the list of devices.
fn device_name() -> String {
    ["COMPUTERNAME", "HOSTNAME"]
        .iter()
        .find_map(|name| std::env::var(name).ok())
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Botloft".to_owned())
}

impl Cloud {
    /// Asks for the link and waits for it in the background. Another
    /// sign-in waiting is dropped. Answers with the seconds the link works.
    pub async fn start_signin(&self, signin: Signin) -> Result<u32, CloudError> {
        let server = self.server()?;
        let started = server
            .login(&signin.email, &device_name(), &signin.locale)
            .await?;
        self.cancel_signin();
        let (url, poll, pending) = (
            self.settings.url.clone(),
            self.settings.poll,
            std::sync::Arc::clone(&self.pending),
        );
        let wait = Duration::from_secs(u64::from(started.wait));
        let task = tokio::spawn(async move {
            let result = wait_for_link(&server, &started.request, wait, poll).await;
            pending
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take();
            match result {
                Some(approved) => {
                    let credentials = Credentials {
                        url,
                        token: approved.token,
                        email: approved.email.clone(),
                        device: approved.device,
                    };
                    if creds::save(&signin.secrets, &credentials).is_ok() {
                        let _ = signin.events.send(Event::CloudSignedIn(CloudSignedIn {
                            email: approved.email,
                        }));
                    } else {
                        tracing::error!("could not keep the sign-in on this computer");
                        let _ = signin.events.send(Event::CloudSigninExpired);
                    }
                }
                None => {
                    let _ = signin.events.send(Event::CloudSigninExpired);
                }
            }
        });
        if !task.is_finished() {
            *self
                .pending
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(task.abort_handle());
        }
        Ok(started.wait)
    }
}

struct Approved {
    token: String,
    email: String,
    device: String,
}

/// Asks every `poll` until the link is opened, or `wait` is over, or the
/// server says it is no use. Network trouble in between is just waiting.
async fn wait_for_link(
    server: &Server,
    request: &str,
    wait: Duration,
    poll: Duration,
) -> Option<Approved> {
    let deadline = Instant::now() + wait;
    while Instant::now() < deadline {
        tokio::time::sleep(poll).await;
        match server.poll(request).await {
            Ok(Polled::Approved {
                token,
                email,
                device,
            }) => {
                return Some(Approved {
                    token,
                    email,
                    device,
                });
            }
            Ok(Polled::Expired) => return None,
            Ok(Polled::Pending) | Err(_) => {}
        }
    }
    None
}
