//! The main window, which starts hidden (spec 15.2): shown at once when the
//! owner opens Botloft, and left near the clock when Windows opens it at
//! sign-in, unless the owner wants the window then too.

use tauri::{AppHandle, Manager as _};

/// What the app hears when Botloft is opened again while it runs, from the
/// Start menu or a notification: it may show what the notification was about.
#[cfg(not(debug_assertions))]
const REOPENED: &str = "botloft://reopened";

pub fn bring_forward(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Botloft was opened again: the running app comes forward instead. Only
/// in a release build, the only one kept to a single instance.
#[cfg(not(debug_assertions))]
pub fn reopened(app: &AppHandle) {
    use tauri::Emitter as _;
    bring_forward(app);
    let _ = app.emit(REOPENED, ());
}
