//! Whether Claude Code is signed in (spec 7.3), from `claude auth status`:
//! checked once Claude Code is found, every 30 s while signed out or
//! unknown, and at once when a bot hits a sign-in error or the app asks.
//! When it turns from signed out to signed in, the bots a sign-in error
//! stopped start again by themselves: the owner just signed in.

use botloft_core::protocol::BotState;
use tokio::time::Instant;
use tracing::{debug, info};

use super::{ClaudeStatus, Inner, REPROBE_AFTER, Supervisor};

#[derive(Debug, Default)]
pub(super) struct SignIn {
    known: Option<bool>,
    checked_at: Option<Instant>,
    recheck: bool,
}

impl SignIn {
    fn due(&self) -> bool {
        if self.recheck {
            return true;
        }
        match (self.known, self.checked_at) {
            (_, None) => true,
            (Some(true), Some(_)) => false,
            (_, Some(at)) => at.elapsed() >= REPROBE_AFTER,
        }
    }
}

impl Supervisor {
    /// Whether Claude Code is signed in; `None` until checked.
    pub fn claude_signed_in(&self) -> Option<bool> {
        self.lock().sign_in.known
    }

    /// Checks Claude Code again now: the sign-in, and the executable if it
    /// was missing or too old (after the owner signed in or installed it).
    pub fn refresh_claude(&self) {
        let mut inner = self.lock();
        inner.sign_in.recheck = true;
        if matches!(inner.claude, ClaudeStatus::Failed { .. }) {
            inner.claude = ClaudeStatus::Unknown;
        }
        drop(inner);
        self.wake();
    }

    /// A bot hit a sign-in error: find out whether Claude Code signed out,
    /// or whether the account itself cannot be used (billing, a blocked
    /// organization), which signing in again does not fix.
    pub(super) fn recheck_sign_in(&self) {
        self.lock().sign_in.recheck = true;
        self.wake();
    }

    pub(super) async fn ensure_sign_in(&self) {
        let program = {
            let inner = self.lock();
            match &inner.claude {
                ClaudeStatus::Ready(claude) if inner.sign_in.due() => claude.path.clone(),
                _ => return,
            }
        };
        let checked = self.runtime.signed_in(program).await;
        let mut inner = self.lock();
        inner.sign_in.recheck = false;
        inner.sign_in.checked_at = Some(Instant::now());
        match checked {
            Ok(signed_in) => {
                let was = inner.sign_in.known.replace(signed_in);
                if signed_in && was == Some(false) {
                    info!("Claude Code is signed in again; starting the bots it stopped");
                    self.start_signed_out_bots(&mut inner);
                }
            }
            // Not fatal: bots still run, and a sign-in error shows in their chat.
            Err(err) => debug!("cannot check the Claude Code sign-in: {err}"),
        }
    }

    fn start_signed_out_bots(&self, inner: &mut Inner) {
        for (bot, slot) in &mut inner.slots {
            if slot.state == BotState::AuthError && slot.running.is_none() {
                slot.backoff.reset();
                slot.restart_at = None;
                self.set_state(bot, slot, BotState::Offline);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_until_signed_in_and_again_when_asked() {
        let mut sign_in = SignIn::default();
        assert!(sign_in.due(), "never checked");
        sign_in.checked_at = Some(Instant::now());
        sign_in.known = Some(false);
        assert!(!sign_in.due(), "checked a moment ago");
        sign_in.known = Some(true);
        assert!(!sign_in.due(), "signed in: nothing to watch");
        sign_in.recheck = true;
        assert!(sign_in.due());
    }
}
