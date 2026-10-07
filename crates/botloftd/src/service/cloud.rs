//! `cloud.*` (spec 27.5). These wait for the network, so the connection
//! answers them aside (`rpc::ASIDE`) and the app is never held up.

use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use botloft_core::protocol::{
    BackupScope, CloudCopies, CloudCopy, CloudCopyParams, CloudDirection, CloudDownloaded,
    CloudProgress, CloudSigninParams, CloudSigninStarted, CloudStatus, CloudUploadParams,
};
use tracing::info;

use super::{ApiError, ApiResult, backup};
use crate::backup::export;
use crate::backup::fingerprint::light_fingerprint;
use crate::cloud::signin::Signin;
use crate::cloud::{self, CloudError, Credentials, Progress, Server};
use crate::state::{Daemon, Event};

/// How long `cloud.status` waits for the server's numbers.
const STATUS_WAIT: Duration = Duration::from_secs(5);

/// The service runs on the blocking pool, where the network waits belong.
fn run<T>(work: impl Future<Output = T>) -> T {
    tokio::runtime::Handle::current().block_on(work)
}

/// A refused token ends the sign-in on this computer: it was revoked.
fn fail(daemon: &Daemon, err: CloudError) -> ApiError {
    if err.reason == "unauthorized" {
        daemon.cloud.forget(&daemon.paths.secrets());
        return ApiError::Rule {
            reason: "signed_out",
            message: "sign in again to use the account".to_owned(),
        };
    }
    ApiError::Rule {
        reason: err.reason,
        message: err.message,
    }
}

pub(super) fn signed_in(daemon: &Daemon) -> ApiResult<(Server, Credentials)> {
    let server = daemon.cloud.server().map_err(|err| fail(daemon, err))?;
    let credentials = daemon
        .cloud
        .credentials(&daemon.paths.secrets())
        .ok_or_else(|| fail(daemon, CloudError::not_signed_in()))?;
    Ok((server, credentials))
}

/// Tells the apps how a transfer goes, at most every 100 ms.
fn progress(daemon: &Daemon, direction: CloudDirection) -> Progress {
    let events = daemon.events();
    let last = Mutex::new(Instant::now() - Duration::from_secs(1));
    Arc::new(move |sent, total| {
        let mut last = last.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if sent >= total || last.elapsed() >= Duration::from_millis(100) {
            *last = Instant::now();
            let _ = events.send(Event::CloudProgress(CloudProgress {
                direction,
                sent,
                total,
            }));
        }
    })
}

/// `cloud.status`
pub fn status(daemon: &Daemon) -> ApiResult<CloudStatus> {
    let mut status = CloudStatus {
        url: daemon.cloud.url().to_owned(),
        signed_in: false,
        email: None,
        used: None,
        quota: None,
        pending: daemon.cloud.is_pending(),
    };
    let Some(credentials) = daemon.cloud.credentials(&daemon.paths.secrets()) else {
        return Ok(status);
    };
    status.signed_in = true;
    status.email = Some(credentials.email.clone());
    let Ok(server) = daemon.cloud.server() else {
        return Ok(status);
    };
    match run(server.me(&credentials.token, STATUS_WAIT)) {
        Ok(me) => {
            status.email = Some(me.email);
            status.used = Some(me.used);
            status.quota = Some(me.quota);
        }
        // Revoked from another computer.
        Err(err) if err.reason == "unauthorized" => {
            let _ = fail(daemon, err);
            status.signed_in = false;
            status.email = None;
        }
        // Offline or slow: the screen still shows who is signed in.
        Err(_) => {}
    }
    Ok(status)
}

/// `cloud.signin`
pub fn signin(daemon: &Daemon, params: CloudSigninParams) -> ApiResult<CloudSigninStarted> {
    let locale = params
        .locale
        .filter(|locale| !locale.trim().is_empty())
        .unwrap_or_else(|| daemon.desktop.locale());
    let wait = run(daemon.cloud.start_signin(Signin {
        email: params.email.trim().to_owned(),
        locale,
        secrets: daemon.paths.secrets(),
        events: daemon.events(),
    }))
    .map_err(|err| fail(daemon, err))?;
    Ok(CloudSigninStarted { wait })
}

/// `cloud.signin_cancel`
pub fn signin_cancel(daemon: &Daemon) {
    daemon.cloud.cancel_signin();
}

/// `cloud.signout`: the server is told if it can be, and this computer forgets
/// either way.
pub fn signout(daemon: &Daemon) {
    daemon.cloud.cancel_signin();
    if let Ok((server, credentials)) = signed_in(daemon) {
        let _ = run(server.logout(&credentials.token));
    }
    daemon.cloud.forget(&daemon.paths.secrets());
    // Without the account there is nowhere to send the copies.
    daemon.autobackup.turn_off();
    daemon.emit(Event::AutoBackupChanged(daemon.autobackup.status()));
}

/// `cloud.upload`: a light copy sealed with the passphrase, sent as the
/// account's newest.
pub fn upload(daemon: &Daemon, params: CloudUploadParams) -> ApiResult<CloudCopy> {
    // What the copy holds, taken first: the automatic backup then knows this
    // one covers it and does not send the same thing again soon.
    let covers = daemon
        .autobackup
        .snapshot()
        .enabled
        .then(|| light_fingerprint(daemon).ok())
        .flatten();
    let copy = send_light(daemon, &params.passphrase)?;
    if let Some(fingerprint) = covers {
        daemon
            .autobackup
            .covered(fingerprint, daemon.clock.now_ms());
    }
    Ok(copy)
}

/// Makes a light copy sealed with `passphrase` and sends it, one at a time.
/// The owner's button and the automatic backup both come here.
pub(crate) fn send_light(daemon: &Daemon, passphrase: &str) -> ApiResult<CloudCopy> {
    let (server, credentials) = signed_in(daemon)?;
    let _sending = daemon.cloud.sending();
    let exported = export::export(daemon, passphrase, BackupScope::Light).map_err(backup::api)?;
    let sent = run(cloud::upload(
        &server,
        &credentials.token,
        std::path::Path::new(&exported.path),
        progress(daemon, CloudDirection::Upload),
    ));
    // The sealed file is not kept: the server has it, or it failed and the
    // next try makes a new one.
    if let Some(folder) = std::path::Path::new(&exported.path).parent() {
        let _ = std::fs::remove_dir_all(folder);
    }
    let copy = sent.map_err(|err| fail(daemon, err))?;
    info!(bytes = copy.size, "a copy went up to the cloud");
    Ok(copy)
}

/// `cloud.copies`
pub fn copies(daemon: &Daemon) -> ApiResult<CloudCopies> {
    let (server, credentials) = signed_in(daemon)?;
    let copies = run(server.copies(&credentials.token)).map_err(|err| fail(daemon, err))?;
    Ok(CloudCopies { copies })
}

/// `cloud.download`: the copy comes to `<home>\cloud-download`, and the app
/// goes on with `backup.stage`. (It cannot be `restore`: staging empties it.)
pub fn download(daemon: &Daemon, params: CloudCopyParams) -> ApiResult<CloudDownloaded> {
    let (server, credentials) = signed_in(daemon)?;
    let folder = daemon.paths.home.join("cloud-download");
    let _ = std::fs::remove_dir_all(&folder);
    let dest = folder.join("copy.botloft");
    run(cloud::download(
        &server,
        &credentials.token,
        &params.id,
        &dest,
        progress(daemon, CloudDirection::Download),
    ))
    .map_err(|err| fail(daemon, err))?;
    Ok(CloudDownloaded {
        path: dest.display().to_string(),
    })
}

/// `cloud.delete`
pub fn delete(daemon: &Daemon, params: CloudCopyParams) -> ApiResult<()> {
    let (server, credentials) = signed_in(daemon)?;
    run(server.delete_copy(&credentials.token, &params.id)).map_err(|err| fail(daemon, err))
}

/// `cloud.delete_account`: the server mails a link; nothing is deleted until
/// the owner presses the button there.
pub fn delete_account(daemon: &Daemon) -> ApiResult<()> {
    let (server, credentials) = signed_in(daemon)?;
    run(server.delete_account(&credentials.token, &daemon.desktop.locale()))
        .map_err(|err| fail(daemon, err))
}
