//! The window people see when they install Botloft (spec 15.7): the mascot,
//! the name and one button. The NSIS installer this executable carries
//! runs quietly behind it and does the installing.

// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod installed;
mod payload;
mod processes;
mod version;

use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use installed::APP_EXE;
use payload::Failure;
use version::Relation;

/// What the window shows before anything happens.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetupState {
    /// The version this setup installs.
    version: &'static str,
    /// No installer inside: the pages only pretend.
    rehearsal: bool,
    /// Where Botloft goes.
    folder: Option<String>,
    /// The version already installed.
    installed: Option<String>,
    relation: Relation,
    /// The app is open from its folder and will close to install.
    app_running: bool,
}

fn state() -> SetupState {
    let installed = installed::read();
    let folder = installed::folder(&installed);
    let version = env!("CARGO_PKG_VERSION");
    SetupState {
        version,
        rehearsal: payload::rehearsal(),
        relation: version::relation(installed.version.as_deref(), version),
        app_running: folder
            .as_deref()
            .is_some_and(|folder| processes::running(folder, APP_EXE)),
        folder: folder.map(|folder| folder.display().to_string()),
        installed: installed.version,
    }
}

#[tauri::command]
async fn setup_state() -> Result<SetupState, String> {
    tauri::async_runtime::spawn_blocking(state)
        .await
        .map_err(|err| err.to_string())
}

/// Closes the open app, then runs the installer quietly and waits for it.
#[tauri::command]
async fn setup_install() -> Result<(), Failure> {
    tauri::async_runtime::spawn_blocking(|| {
        if payload::rehearsal() {
            std::thread::sleep(Duration::from_secs(3));
            return Ok(());
        }
        if let Some(folder) = installed::folder(&installed::read()) {
            processes::close(&folder, APP_EXE);
        }
        payload::install_quietly()
    })
    .await
    .map_err(|err| Failure {
        code: None,
        detail: err.to_string(),
    })?
}

/// Opens the installed app and closes the setup.
#[tauri::command]
fn setup_open_app(app: AppHandle) -> Result<(), String> {
    if !payload::rehearsal() {
        let folder = installed::read()
            .folder
            .ok_or("Botloft is not in Installed apps")?;
        std::process::Command::new(folder.join(APP_EXE))
            .current_dir(&folder)
            .spawn()
            .map_err(|err| format!("cannot open {APP_EXE}: {err}"))?;
    }
    app.exit(0);
    Ok(())
}

/// Hides this window and runs the installer with its own pages, then
/// closes: for when the quiet install failed.
#[tauri::command]
async fn setup_classic(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    let ran = tauri::async_runtime::spawn_blocking(|| {
        if payload::rehearsal() {
            Ok(())
        } else {
            payload::install_classic()
        }
    })
    .await;
    app.exit(0);
    // The classic installer speaks for itself; a cancel there is no error.
    ran.map(drop).map_err(|err| err.to_string())
}

fn main() {
    // Without WebView2 this window cannot open. The classic installer has
    // its own pages and fetches WebView2 (spec 15.7).
    if tauri::webview_version().is_err() {
        if !payload::rehearsal() {
            let _ = payload::install_classic();
        }
        return;
    }
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            setup_state,
            setup_install,
            setup_open_app,
            setup_classic
        ])
        .setup(|app| {
            // WebView2 keeps its files under %TEMP%, reused each time, and
            // not in a folder of the setup's own under %LOCALAPPDATA%.
            let data = std::env::temp_dir().join("Botloft-setup-webview");
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("setup.html".into()))
                .title("Botloft")
                .inner_size(480.0, 380.0)
                .resizable(false)
                .maximizable(false)
                .decorations(false)
                .center()
                .visible(false)
                .data_directory(data)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run the setup window");
}
