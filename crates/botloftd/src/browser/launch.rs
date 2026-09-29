//! Starting a bot's browser (spec 21.2): find Edge, run it without a window
//! in a job of its own, and read where its DevTools listen.

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use tracing::debug;

use super::BrowserError;
use crate::platform::{self, ProcessJob};

/// How long Edge may take to write `DevToolsActivePort`.
const START_TIMEOUT: Duration = Duration::from_secs(10);
const PORT_FILE: &str = "DevToolsActivePort";

/// The browser's process tree. Dropping it kills every process in it.
pub struct BrowserProcess {
    child: Child,
    job: ProcessJob,
}

impl BrowserProcess {
    pub fn kill(&mut self) {
        let _ = self.job.terminate();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        self.kill();
    }
}

/// The browser to run: `configured`, or the Edge that comes with Windows.
pub fn find(configured: &str) -> Option<PathBuf> {
    let configured = configured.trim();
    if !configured.is_empty() {
        let path = PathBuf::from(configured);
        return path.is_file().then_some(path);
    }
    ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"]
        .iter()
        .filter_map(std::env::var_os)
        .map(|base| {
            PathBuf::from(base)
                .join("Microsoft")
                .join("Edge")
                .join("Application")
                .join("msedge.exe")
        })
        .find(|path| path.is_file())
}

/// Starts `program` with the profile in `profile` and returns it with the
/// DevTools WebSocket address.
pub async fn launch(
    program: &Path,
    profile: &Path,
) -> Result<(BrowserProcess, String), BrowserError> {
    std::fs::create_dir_all(profile)?;
    let port_file = profile.join(PORT_FILE);
    match std::fs::remove_file(&port_file) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err.into()),
    }
    let env = platform::user_environment()?;
    let mut command = Command::new(program);
    command
        .args([
            "--headless=new",
            "--remote-debugging-port=0",
            "--no-first-run",
            "--no-default-browser-check",
            "--mute-audio",
            // Extensions installed for every profile on the computer would
            // open their own tabs in the bot's browser.
            "--disable-extensions",
            "--disable-sync",
        ])
        .arg(format!("--user-data-dir={}", profile.display()))
        .arg("about:blank")
        .env_clear()
        .envs(env.iter().map(|(name, value)| (name, value)))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let job = ProcessJob::new()?;
    let child = command.spawn()?;
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle as _;
        job.assign(child.as_raw_handle())?;
    }
    let mut process = BrowserProcess { child, job };
    debug!(pid = process.child.id(), "browser: started");

    let started = Instant::now();
    loop {
        if let Some(url) = std::fs::read_to_string(&port_file)
            .ok()
            .and_then(|text| devtools_url(&text))
        {
            return Ok((process, url));
        }
        if process.child.try_wait()?.is_some() {
            return Err(BrowserError::Start(
                "the browser closed right after starting".to_owned(),
            ));
        }
        if started.elapsed() > START_TIMEOUT {
            return Err(BrowserError::Start(
                "the browser did not start in time".to_owned(),
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// `DevToolsActivePort` holds the port and the browser's WebSocket path, one
/// per line. The file may be read half-written.
fn devtools_url(text: &str) -> Option<String> {
    let mut lines = text.lines();
    let port: u16 = lines.next()?.trim().parse().ok().filter(|port| *port > 0)?;
    let path = lines.next()?.trim();
    path.starts_with("/devtools/browser/")
        .then(|| format!("ws://127.0.0.1:{port}{path}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_port_file() {
        assert_eq!(
            devtools_url("62475\n/devtools/browser/c75e9f8f\n").as_deref(),
            Some("ws://127.0.0.1:62475/devtools/browser/c75e9f8f")
        );
        assert_eq!(devtools_url("62475\n"), None);
        assert_eq!(devtools_url("0\n/devtools/browser/x"), None);
        assert_eq!(devtools_url(""), None);
    }

    #[test]
    fn a_configured_browser_must_exist() {
        assert_eq!(find(r"C:\nowhere\msedge.exe"), None);
    }
}
