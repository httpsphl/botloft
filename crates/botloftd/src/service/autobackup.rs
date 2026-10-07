//! `autobackup.*` (spec 27.10): turning the automatic backup on and off. The
//! sending itself is `crate::autobackup`.

use botloft_core::protocol::{AutoBackupEnableParams, AutoBackupEveryParams, AutoBackupStatus};

use super::{ApiError, ApiResult, backup, cloud};
use crate::autobackup::Saved;
use crate::backup::seal::{PASSPHRASE_MIN, SealError};
use crate::state::{Daemon, Event};

fn rule(reason: &'static str, message: &str) -> ApiError {
    ApiError::Rule {
        reason,
        message: message.to_owned(),
    }
}

fn changed(daemon: &Daemon) -> AutoBackupStatus {
    let status = daemon.autobackup.status();
    daemon.emit(Event::AutoBackupChanged(status.clone()));
    status
}

/// `autobackup.status`
pub fn status(daemon: &Daemon) -> AutoBackupStatus {
    daemon.autobackup.status()
}

/// `autobackup.enable`: keeps the passphrase in the credential store and
/// looks at once.
pub fn enable(daemon: &Daemon, params: AutoBackupEnableParams) -> ApiResult<AutoBackupStatus> {
    if !daemon.autobackup.secrets().available() {
        return Err(rule(
            "no_keystore",
            "this computer has nowhere safe to keep the passphrase",
        ));
    }
    if params.passphrase.chars().count() < PASSPHRASE_MIN {
        return Err(backup::api(SealError::ShortPassphrase));
    }
    // The copies go to the account, so it has to be there.
    cloud::signed_in(daemon)?;
    daemon
        .autobackup
        .secrets()
        .save(&params.passphrase)
        .map_err(|_| rule("no_keystore", "the passphrase could not be kept safely"))?;
    daemon.autobackup.update(|saved| {
        *saved = Saved {
            enabled: true,
            every: params.every,
            last_ok_at: saved.last_ok_at,
            ..Saved::default()
        };
    });
    daemon.autobackup.wake();
    Ok(changed(daemon))
}

/// `autobackup.set_every`
pub fn set_every(daemon: &Daemon, params: AutoBackupEveryParams) -> ApiResult<AutoBackupStatus> {
    daemon.autobackup.update(|saved| {
        saved.every = params.every;
        // Looks again soon, finds nothing new if so, and counts the new period.
        saved.next_at = None;
    });
    daemon.autobackup.wake();
    Ok(changed(daemon))
}

/// `autobackup.disable`
pub fn disable(daemon: &Daemon) -> AutoBackupStatus {
    daemon.autobackup.turn_off();
    changed(daemon)
}
