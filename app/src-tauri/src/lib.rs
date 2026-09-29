//! Tauri shell for the Botloft app. The UI talks to `botloftd` over
//! WebSocket; these commands cover what a web page cannot do (spec 15.2).

mod claude;
mod daemon;
mod sign_in;
mod window;

use std::path::Path;

use daemon::DaemonStatus;

/// Runs blocking work off the main thread.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|err| err.to_string())?
}

/// Whether the daemon answers `/health`, and on which port.
#[tauri::command]
async fn daemon_status() -> Result<DaemonStatus, String> {
    blocking(|| daemon::endpoint().map(|endpoint| daemon::status(&endpoint))).await
}

/// Installs the daemon this app ships as a scheduled task and starts it,
/// replacing an older one (spec 14).
#[tauri::command]
async fn daemon_install() -> Result<DaemonStatus, String> {
    blocking(|| daemon::install(&daemon::endpoint()?)).await
}

/// Stops the daemon and starts it again from its scheduled task.
#[tauri::command]
async fn daemon_restart() -> Result<DaemonStatus, String> {
    blocking(|| daemon::restart(&daemon::endpoint()?)).await
}

/// Stops the daemon until the app opens again, or until the next sign-in
/// when it starts with Windows: for an owner who wants the bots to stop
/// when they close Botloft.
#[tauri::command]
async fn daemon_stop() -> Result<(), String> {
    blocking(|| daemon::stop(&daemon::endpoint()?)).await
}

/// Opens Claude Code's sign-in in its own window and waits for it to end.
/// Returns whether it signed in.
#[tauri::command]
async fn claude_sign_in(path: String) -> Result<bool, String> {
    blocking(move || claude::sign_in(Path::new(&path))).await
}

#[tauri::command]
async fn read_owner_token() -> Result<String, String> {
    blocking(|| daemon::owner_token(&daemon::endpoint()?)).await
}

/// Opens a folder in Explorer. Files are refused: Explorer would run them.
#[tauri::command]
async fn open_path(path: String) -> Result<(), String> {
    blocking(move || {
        let folder = Path::new(&path);
        if !folder.is_absolute() || !folder.is_dir() {
            return Err(format!("{path} is not a folder"));
        }
        std::process::Command::new("explorer.exe")
            .arg(folder)
            .spawn()
            .map(drop)
            .map_err(|err| format!("cannot open Explorer: {err}"))
    })
    .await
}

/// Extensions `open_file` will open with the program Windows picks. Anything
/// else could be a program or a script that runs when opened.
const OPENABLE: [&str; 24] = [
    "pdf", "png", "jpg", "jpeg", "gif", "webp", "svg", "txt", "md", "csv", "json", "log", "html",
    "htm", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "mp3", "wav", "mp4", "rtf",
];

fn is_openable(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| OPENABLE.contains(&extension.to_ascii_lowercase().as_str()))
}

/// Opens a file a bot made with the program Windows uses for it. Only
/// documents, images, sound and video: never something that runs.
#[tauri::command]
async fn open_file(path: String) -> Result<(), String> {
    blocking(move || {
        let file = Path::new(&path);
        if !file.is_absolute() || !file.is_file() {
            return Err(format!("{path} is not a file"));
        }
        if !is_openable(file) {
            return Err("this kind of file is not opened from Botloft".into());
        }
        std::process::Command::new("explorer.exe")
            .arg(file)
            .spawn()
            .map(drop)
            .map_err(|err| format!("cannot open the file: {err}"))
    })
    .await
}

/// Shows a file in its folder, selected. Nothing is run.
#[tauri::command]
async fn reveal_file(path: String) -> Result<(), String> {
    use std::os::windows::process::CommandExt as _;
    blocking(move || {
        let file = Path::new(&path);
        if !file.is_absolute() || !file.exists() || path.contains('"') {
            return Err(format!("{path} is not a file"));
        }
        // Explorer reads `/select,` and the path as one argument.
        std::process::Command::new("explorer.exe")
            .raw_arg(format!("/select,\"{path}\""))
            .spawn()
            .map(drop)
            .map_err(|err| format!("cannot open Explorer: {err}"))
    })
    .await
}

/// Opens Botloft when the owner signs in to Windows, or stops doing so.
#[tauri::command]
async fn open_at_sign_in(on: bool) -> Result<(), String> {
    blocking(move || sign_in::set(on)).await
}

/// Whether Windows opened this app at sign-in, with its window hidden.
#[tauri::command]
fn launched_at_sign_in() -> bool {
    sign_in::launched_at_sign_in()
}

/// Whether `url` is a plain web link, safe to hand to the default browser.
fn is_web_link(url: &str) -> bool {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    rest.is_some_and(|rest| !rest.is_empty())
        && url.len() <= 2048
        && !url
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control() || ch == '"')
}

/// Opens a web link from a bot's reply in the default browser. Only
/// `http` and `https`: anything else could start a program.
#[tauri::command]
async fn open_url(url: String) -> Result<(), String> {
    if !is_web_link(&url) {
        return Err("only web links can be opened".into());
    }
    blocking(move || {
        std::process::Command::new("explorer.exe")
            .arg(&url)
            .spawn()
            .map(drop)
            .map_err(|err| format!("cannot open the browser: {err}"))
    })
    .await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();
    // Opening Botloft again brings the running one forward. Not in a dev
    // build, which runs beside the installed app.
    #[cfg(not(debug_assertions))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        window::reopened(app);
    }));
    builder
        // Checks the release feed and installs signed updates (spec 15.5).
        .plugin(tauri_plugin_updater::Builder::new().build())
        // The folder picker for a crew's work folder (spec 5).
        .plugin(tauri_plugin_dialog::init())
        // Windows notifications when a bot needs the owner (spec 15.1).
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            if !sign_in::launched_at_sign_in() {
                window::bring_forward(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            daemon_status,
            daemon_install,
            daemon_restart,
            daemon_stop,
            claude_sign_in,
            read_owner_token,
            open_path,
            open_file,
            reveal_file,
            open_url,
            open_at_sign_in,
            launched_at_sign_in
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Botloft app");
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{is_openable, is_web_link};

    #[test]
    fn only_documents_and_media_open() {
        for file in [r"C:\w\Report.PDF", r"C:\w\a b\notes.md", r"C:\w\chart.png"] {
            assert!(is_openable(Path::new(file)), "{file}");
        }
        for file in [
            r"C:\w\setup.exe",
            r"C:\w\run.bat",
            r"C:\w\run.cmd",
            r"C:\w\run.ps1",
            r"C:\w\link.lnk",
            r"C:\w\page.hta",
            r"C:\w\macro.docm",
            r"C:\w\noextension",
            r"C:\w\report.pdf.exe",
        ] {
            assert!(!is_openable(Path::new(file)), "{file}");
        }
    }

    #[test]
    fn only_plain_web_links_open() {
        assert!(is_web_link("https://code.claude.com/docs"));
        assert!(is_web_link("http://127.0.0.1:3000/page?a=1&b=2"));
        for url in [
            "file:///C:/Windows/System32/calc.exe",
            r"C:\Windows\notepad.exe",
            "https://",
            "https://a b",
            "https://a\"b",
            "ms-settings:privacy",
        ] {
            assert!(!is_web_link(url), "{url}");
        }
    }
}
