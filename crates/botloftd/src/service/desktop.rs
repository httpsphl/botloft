//! What each bot may see and do on the owner's desktop (spec 24.2,
//! 24.10): `desktop.grants` and `desktop.revoke`. Grants come from the
//! owner answering a bot's request (`tools/desktop.rs`), or from Settings
//! for an app open now (`desktop.apps`, `desktop.grantApp`). And stopping
//! a bot there, or every bot at once (spec 24.9): `desktop.stop`,
//! `desktop.resume` and `desktop.stopAll`.

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    BotDesktop, DesktopApp, DesktopAwayUse, DesktopBotParams, DesktopGrant, DesktopGrantAppParams,
    DesktopGrantIdParams, DesktopGrantsParams, DesktopOptionsParams, DesktopState,
    DesktopWholeParams,
};
use tracing::{info, warn};

use super::{ApiError, ApiResult};
use crate::platform::desktop::{self as platform, DesktopError, Window, never};
use crate::state::{Daemon, Event};

pub fn grants(daemon: &Daemon, params: DesktopGrantsParams) -> ApiResult<Vec<DesktopGrant>> {
    let store = daemon.store();
    if store.bot(&params.bot_id)?.is_none() {
        return Err(ApiError::NotFound(format!("bot {}", params.bot_id)));
    }
    Ok(store.desktop_grants(&params.bot_id)?)
}

/// Changes a grant's options: the real mouse and keyboard, and use while
/// the owner is away, which only goes on with the risks accepted (spec
/// 24.10).
pub fn set_options(daemon: &Daemon, params: DesktopOptionsParams) -> ApiResult<BotDesktop> {
    if params.unattended == Some(true) && params.accepted_risks != Some(true) {
        return Err(ApiError::Conflict(
            "use while the owner is away needs the risks accepted first".to_owned(),
        ));
    }
    let store = daemon.store();
    let bot_id = store
        .desktop_grants_of(&params.grant_id)?
        .ok_or_else(|| ApiError::NotFound(format!("desktop grant {}", params.grant_id)))?;
    if let Some(on) = params.real_input {
        store.set_desktop_real_input(&params.grant_id, on)?;
    }
    if let Some(on) = params.unattended {
        store.set_desktop_unattended(&params.grant_id, on, daemon.clock.now_ms())?;
        info!(on, "use of the desktop while the owner is away changed");
    }
    drop(store);
    changed(daemon, &bot_id)
}

/// Gives the bot the whole desktop at a level, only with the risks
/// accepted (spec 24.10).
pub fn grant_whole(daemon: &Daemon, params: DesktopWholeParams) -> ApiResult<BotDesktop> {
    if !params.accepted_risks {
        return Err(ApiError::Conflict(
            "the whole desktop needs the risks accepted first".to_owned(),
        ));
    }
    super::bots::find(&daemon.store(), &params.bot_id)?;
    daemon
        .store()
        .grant_desktop_whole(&params.bot_id, params.level, daemon.clock.now_ms())?;
    info!(
        level = params.level.as_str(),
        "the owner gave a bot the whole desktop"
    );
    changed(daemon, &params.bot_id)
}

/// The apps open now that a bot may be given (spec 24.10): one per
/// executable, by name, never one of those never granted (24.3). Empty
/// where there is no desktop to use.
pub fn apps() -> ApiResult<Vec<DesktopApp>> {
    Ok(grantable(&open_windows()))
}

fn open_windows() -> Vec<Window> {
    match platform::windows() {
        Ok(windows) => windows,
        Err(DesktopError::Unavailable) => Vec::new(),
        Err(err) => {
            warn!("could not list the open windows: {err}");
            Vec::new()
        }
    }
}

fn grantable(windows: &[Window]) -> Vec<DesktopApp> {
    let mut apps: Vec<DesktopApp> = Vec::new();
    for window in windows {
        let path = window.app.path.to_string_lossy();
        let known = apps.iter().any(|app| app.path.eq_ignore_ascii_case(&path));
        if path.is_empty() || known || never(window).is_some() {
            continue;
        }
        apps.push(DesktopApp {
            path: path.into_owned(),
            name: window.app.name.clone(),
        });
    }
    apps.sort_by_key(|app| app.name.to_lowercase());
    apps
}

/// Gives the bot an app that is open now, before it asks (spec 24.10):
/// the daemon names it from its windows and refuses one never granted.
pub fn grant_app(daemon: &Daemon, params: DesktopGrantAppParams) -> ApiResult<BotDesktop> {
    super::bots::find(&daemon.store(), &params.bot_id)?;
    let windows = open_windows();
    let open: Vec<&Window> = windows
        .iter()
        .filter(|window| {
            window
                .app
                .path
                .to_string_lossy()
                .eq_ignore_ascii_case(&params.path)
        })
        .collect();
    if open.is_empty() {
        return Err(ApiError::NotFound(
            "that app is not open on the desktop".to_owned(),
        ));
    }
    let Some(window) = open.into_iter().find(|window| never(window).is_none()) else {
        return Err(ApiError::Conflict(
            "that app is never given to a bot".to_owned(),
        ));
    };
    let path = window.app.path.to_string_lossy().into_owned();
    daemon.store().grant_desktop_app(
        &params.bot_id,
        &path,
        &window.app.name,
        params.level,
        daemon.clock.now_ms(),
    )?;
    info!(
        level = params.level.as_str(),
        "the owner gave a bot an app on the desktop"
    );
    changed(daemon, &params.bot_id)
}

/// What bots did while the owner was away, for the app to tell them.
pub fn away_uses(daemon: &Daemon) -> ApiResult<Vec<DesktopAwayUse>> {
    Ok(daemon.desktop.away.list())
}

/// The owner saw what bots did while they were away.
pub fn dismiss_away(daemon: &Daemon) -> ApiResult<()> {
    daemon.desktop.away.clear();
    daemon.emit(Event::DesktopAway(Vec::new()));
    Ok(())
}

/// Takes a grant away: the bot asks for that again.
pub fn revoke(daemon: &Daemon, params: DesktopGrantIdParams) -> ApiResult<BotDesktop> {
    let bot_id = daemon
        .store()
        .revoke_desktop_grant(&params.grant_id)?
        .ok_or_else(|| ApiError::NotFound(format!("desktop grant {}", params.grant_id)))?;
    changed(daemon, &bot_id)
}

/// Tells the apps the bot's grants now.
pub(crate) fn changed(daemon: &Daemon, bot_id: &BotId) -> ApiResult<BotDesktop> {
    let desktop = BotDesktop {
        grants: daemon.store().desktop_grants(bot_id)?,
        bot_id: bot_id.clone(),
    };
    daemon.emit(Event::BotDesktop(desktop.clone()));
    Ok(desktop)
}

/// Stops the bot on the desktop, or lets it go on.
fn set_stopped(daemon: &Daemon, bot: &BotId, stopped: bool) -> DesktopState {
    let state = daemon.desktop.activity.set_stopped(bot, stopped);
    daemon.emit(Event::DesktopChanged(state.clone()));
    state
}

pub fn stop(daemon: &Daemon, params: DesktopBotParams) -> ApiResult<DesktopState> {
    super::bots::find(&daemon.store(), &params.bot_id)?;
    Ok(set_stopped(daemon, &params.bot_id, true))
}

pub fn resume(daemon: &Daemon, params: DesktopBotParams) -> ApiResult<DesktopState> {
    super::bots::find(&daemon.store(), &params.bot_id)?;
    Ok(set_stopped(daemon, &params.bot_id, false))
}

/// Stops every bot on the desktop: the owner pressed the stop shortcut
/// (spec 24.9).
pub fn stop_all(daemon: &Daemon) {
    let bots = match daemon.store().bots(None, false) {
        Ok(bots) => bots,
        Err(err) => {
            warn!("could not list the bots to stop them on the desktop: {err}");
            return;
        }
    };
    for bot in &bots {
        set_stopped(daemon, &bot.id, true);
    }
    info!(
        bots = bots.len(),
        "the owner stopped every bot on the desktop"
    );
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::platform::desktop::App;

    fn window(id: u64, path: &str, name: &str, elevated: bool) -> Window {
        Window {
            id,
            title: format!("Window {id}"),
            class: "Test".to_owned(),
            app: App {
                path: PathBuf::from(path),
                name: name.to_owned(),
            },
            minimized: false,
            elevated,
        }
    }

    #[test]
    fn the_apps_to_give_are_one_per_program_and_never_the_ones_never_granted() {
        let windows = [
            window(1, r"C:\Office\EXCEL.EXE", "Microsoft Excel", false),
            window(2, r"C:\office\excel.exe", "Microsoft Excel", false),
            window(3, r"C:\Windows\System32\cmd.exe", "Command Prompt", false),
            window(4, r"C:\Tools\admin.exe", "Admin tool", true),
            window(5, r"C:\Botloft\Botloft.exe", "Botloft", false),
            window(6, "", "Nameless", false),
            window(7, r"C:\Apps\notepad.exe", "Bloco de Notas", false),
        ];
        let apps = grantable(&windows);
        let names: Vec<&str> = apps.iter().map(|app| app.name.as_str()).collect();
        assert_eq!(names, ["Bloco de Notas", "Microsoft Excel"]);
        assert_eq!(apps[1].path, r"C:\Office\EXCEL.EXE");
    }
}
