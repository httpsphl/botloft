//! Starting a bot's browser (spec 21.2): run it without a window in a job
//! of its own, and read where its DevTools listen.

use std::io::{self, BufRead as _, BufReader};
use std::path::Path;
use std::process::{Child, ChildStderr, Command, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use tracing::debug;

use super::BrowserError;
use crate::platform::{self, ProcessJob};

/// How long Edge may take to write `DevToolsActivePort`.
const START_TIMEOUT: Duration = Duration::from_secs(10);
const PORT_FILE: &str = "DevToolsActivePort";
/// Longest stderr line passed on as the reason a browser stopped.
const REASON_MAX: usize = 300;

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
        .stderr(Stdio::piped());
    ProcessJob::prepare(&mut command);
    let job = ProcessJob::new()?;
    let mut child = command.spawn()?;
    if let Err(err) = job.assign(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(err.into());
    }
    let said = child.stderr.take().map(StartLog::read);
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
            // Its helpers may still hold stderr open until the group dies.
            process.kill();
            if let Some(said) = &said {
                let _ = said.ended.recv_timeout(Duration::from_secs(1));
            }
            return Err(BrowserError::Start(format!(
                "the browser closed right after starting{}",
                StartLog::reason(said.as_ref())
            )));
        }
        if started.elapsed() > START_TIMEOUT {
            return Err(BrowserError::Start(format!(
                "the browser did not start in time{}",
                StartLog::reason(said.as_ref())
            )));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// What the browser writes to stderr, kept as the line that best says why
/// it failed to start (a missing sandbox on Linux, say). Nothing of it goes
/// to the log.
struct StartLog {
    lines: Arc<Mutex<Said>>,
    /// Hears when stderr ends.
    ended: mpsc::Receiver<()>,
}

#[derive(Default)]
struct Said {
    best: Option<String>,
    last: Option<String>,
}

impl StartLog {
    fn read(stderr: ChildStderr) -> Self {
        let lines = Arc::new(Mutex::new(Said::default()));
        let (end, ended) = mpsc::channel();
        let shared = Arc::clone(&lines);
        let spawned = std::thread::Builder::new()
            .name("browser-stderr".into())
            .spawn(move || {
                for line in BufReader::new(stderr).lines() {
                    let Ok(line) = line else { break };
                    let text: String = log_text(&line).chars().take(REASON_MAX).collect();
                    if text.is_empty() {
                        continue;
                    }
                    let lower = line.to_ascii_lowercase();
                    let mut said = shared.lock().unwrap_or_else(|p| p.into_inner());
                    let sandbox = lower.contains("sandbox") || lower.contains("namespace");
                    if sandbox || (said.best.is_none() && lower.contains(":fatal:")) {
                        said.best = Some(text.clone());
                    }
                    said.last = Some(text);
                }
                let _ = end.send(());
            });
        if spawned.is_err() {
            debug!("browser: cannot read its stderr");
        }
        Self { lines, ended }
    }

    /// ` (the line)`, or nothing when the browser said nothing.
    fn reason(log: Option<&Self>) -> String {
        let Some(log) = log else {
            return String::new();
        };
        let said = log.lines.lock().unwrap_or_else(|p| p.into_inner());
        said.best
            .as_ref()
            .or(said.last.as_ref())
            .map(|line| format!(" ({line})"))
            .unwrap_or_default()
    }
}

/// A Chromium log line without its `[pid:tid:time:LEVEL:file.cc:123]` prefix.
fn log_text(line: &str) -> String {
    let text = match line.strip_prefix('[') {
        Some(rest) => rest.split_once("] ").map_or(line, |(_, text)| text),
        None => line,
    };
    text.trim().to_owned()
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
    fn a_log_line_loses_its_prefix() {
        assert_eq!(
            log_text(
                "[11:11:1001/173604.487235:ERROR:zygote_host_impl_linux.cc:129] No usable sandbox!"
            ),
            "No usable sandbox!"
        );
        assert_eq!(log_text("  plain words "), "plain words");
    }
}
