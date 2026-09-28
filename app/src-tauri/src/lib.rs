//! Tauri shell for the Botloft app. The UI talks to `botloftd` over
//! WebSocket; these commands cover what a web page cannot do (spec 15.2).

mod daemon;

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

/// Starts the daemon next to the app when it is not running (M4; M5
/// installs it as a scheduled task instead).
#[tauri::command]
async fn daemon_start() -> Result<DaemonStatus, String> {
    blocking(|| daemon::start(&daemon::endpoint()?)).await
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
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            daemon_status,
            daemon_start,
            read_owner_token,
            open_path,
            open_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Botloft app");
}

#[cfg(test)]
mod tests {
    use super::is_web_link;

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
