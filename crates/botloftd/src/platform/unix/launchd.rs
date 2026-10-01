//! The daemon as a launch agent on macOS (spec 14.1), in
//! `~/Library/LaunchAgents`. `RunAtLoad` starts it when the owner signs
//! in. `KeepAlive` keeps it running while its keep-alive file exists:
//! starting creates the file and stopping deletes it, so launchd brings the
//! daemon back after a crash but not after a stop asked for. A plain
//! `KeepAlive` would also start it at every sign-in.

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use super::write_atomically;
use crate::platform::{TaskDefinition, TaskInfo, TaskState, Triggers};

pub const TASK_KIND: &str = "launch agent";

/// `Botloft` is the installed daemon's agent; `Botloft-<hash>` a dev one.
fn label(name: &str) -> String {
    let rest = name.strip_prefix("Botloft").unwrap_or(name);
    format!("io.github.httpsphl.botloft.daemon{}", rest.to_lowercase())
}

fn plist_path(name: &str) -> io::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| io::Error::other("cannot find the home folder"))?;
    Ok(home
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{}.plist", label(name))))
}

/// The owner's GUI session, where agents run with their sign-in.
fn domain() -> String {
    // SAFETY: no arguments, never fails.
    format!("gui/{}", unsafe { libc::getuid() })
}

fn service(name: &str) -> String {
    format!("{}/{}", domain(), label(name))
}

fn launchctl(args: &[&str]) -> io::Result<Output> {
    Command::new("launchctl")
        .args(args)
        .stdin(Stdio::null())
        .output()
}

fn run(args: &[&str]) -> io::Result<()> {
    let output = launchctl(args)?;
    if output.status.success() {
        return Ok(());
    }
    let said = [output.stderr, output.stdout]
        .map(|bytes| String::from_utf8_lossy(&bytes).trim().to_owned())
        .join(" ");
    Err(io::Error::other(format!(
        "`launchctl {}` failed: {}",
        args.join(" "),
        said.trim()
    )))
}

pub fn register_task(name: &str, task: &TaskDefinition) -> io::Result<()> {
    write_atomically(
        &plist_path(name)?,
        plist_text(&label(name), task).as_bytes(),
    )
}

pub fn find_task(name: &str) -> io::Result<Option<TaskInfo>> {
    let text = match std::fs::read_to_string(plist_path(name)?) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    };
    let state = match launchctl(&["print", &service(name)]) {
        Ok(output) if output.status.success() => {
            if String::from_utf8_lossy(&output.stdout).contains("state = running") {
                TaskState::Running
            } else {
                TaskState::Ready
            }
        }
        // Not loaded: it waits for the next sign-in or for `run_task`.
        Ok(_) => TaskState::Ready,
        Err(_) => TaskState::Unknown,
    };
    let command = strings(&text, "ProgramArguments").map(|args| args.join(" "));
    Ok(Some(TaskInfo {
        state,
        command,
        triggers: Triggers {
            logon: text.contains("<key>RunAtLoad</key><true/>"),
            watchdog: true,
        },
    }))
}

/// Loads the agent anew from its file and starts it.
pub fn run_task(name: &str) -> io::Result<()> {
    let path = plist_path(name)?;
    let text = std::fs::read_to_string(&path)?;
    if let Some(file) = keep_alive(&text) {
        write_atomically(&file, b"")?;
    }
    // Errors only mean it was not loaded.
    let _ = launchctl(&["bootout", &service(name)]);
    let path = path.to_string_lossy();
    // Right after a bootout, launchd may still refuse the same label.
    let mut tries = 0;
    loop {
        match run(&["bootstrap", &domain(), &path]) {
            Ok(()) => break,
            Err(_) if tries < 10 => {
                tries += 1;
                std::thread::sleep(Duration::from_millis(300));
            }
            Err(err) => return Err(err),
        }
    }
    // With the keep-alive file there, launchd may have started it already;
    // whether it answers is what the caller waits for.
    let _ = launchctl(&["kickstart", &service(name)]);
    Ok(())
}

/// Deletes the keep-alive file, then asks the daemon to stop.
pub fn stop_task(name: &str) -> io::Result<()> {
    let text = std::fs::read_to_string(plist_path(name)?)?;
    if let Some(file) = keep_alive(&text) {
        remove(&file)?;
    }
    run(&["kill", "SIGTERM", &service(name)])
}

pub fn delete_task(name: &str) -> io::Result<bool> {
    let path = plist_path(name)?;
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err),
    };
    if let Some(file) = keep_alive(&text) {
        remove(&file)?;
    }
    let _ = launchctl(&["bootout", &service(name)]);
    std::fs::remove_file(&path)?;
    Ok(true)
}

fn remove(path: &Path) -> io::Result<()> {
    match std::fs::remove_file(path) {
        Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
        _ => Ok(()),
    }
}

fn plist_text(label: &str, task: &TaskDefinition) -> String {
    let arguments: String = std::iter::once(task.program.to_string_lossy().into_owned())
        .chain(task.arguments.iter().cloned())
        .map(|arg| format!("    <string>{}</string>\n", escape(&arg)))
        .collect();
    let run_at_load = if task.triggers.logon {
        "<true/>"
    } else {
        "<false/>"
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<!-- Written by Botloft (spec 14.1). `botloftd service uninstall` removes it. -->
<plist version="1.0">
<dict>
  <key>Label</key><string>{label}</string>
  <key>ProgramArguments</key>
  <array>
{arguments}  </array>
  <key>WorkingDirectory</key><string>{dir}</string>
  <key>RunAtLoad</key>{run_at_load}
  <key>KeepAlive</key>
  <dict>
    <key>PathState</key>
    <dict>
      <key>{keep_alive}</key><true/>
    </dict>
  </dict>
  <key>ProcessType</key><string>Standard</string>
</dict>
</plist>
"#,
        label = escape(label),
        dir = escape(&task.working_dir.to_string_lossy()),
        keep_alive = escape(&task.keep_alive.to_string_lossy()),
    )
}

/// The keep-alive file named in the agent this module wrote.
fn keep_alive(plist: &str) -> Option<PathBuf> {
    let rest = plist.split_once("<key>PathState</key>")?.1;
    let key = rest.split_once("<key>")?.1.split_once("</key>")?.0;
    Some(PathBuf::from(unescape(key)))
}

/// The `<string>` values of the array under `key`.
fn strings(plist: &str, key: &str) -> Option<Vec<String>> {
    let rest = plist.split_once(&format!("<key>{key}</key>"))?.1;
    let array = rest.split_once("<array>")?.1.split_once("</array>")?.0;
    Some(
        array
            .split("<string>")
            .skip(1)
            .filter_map(|part| part.split_once("</string>"))
            .map(|(value, _)| unescape(value))
            .collect(),
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(logon: bool) -> TaskDefinition {
        TaskDefinition {
            description: "Starts Botloft".into(),
            program: PathBuf::from("/Users/ana/Library/Application Support/Botloft/bin/botloftd"),
            arguments: ["serve", "--home", "/Users/ana/A & B", "--scheduled"]
                .map(String::from)
                .to_vec(),
            working_dir: PathBuf::from("/Users/ana/A & B"),
            keep_alive: PathBuf::from("/Users/ana/A & B/run/keep-alive"),
            triggers: Triggers {
                logon,
                watchdog: true,
            },
        }
    }

    #[test]
    fn the_agent_runs_the_daemon_while_its_keep_alive_file_exists() {
        let text = plist_text("io.github.httpsphl.botloft.daemon", &task(true));
        assert!(
            text.contains("<key>Label</key><string>io.github.httpsphl.botloft.daemon</string>")
        );
        assert!(text.contains("<key>RunAtLoad</key><true/>"));
        assert!(text.contains("<key>WorkingDirectory</key><string>/Users/ana/A &amp; B</string>"));
        assert_eq!(
            keep_alive(&text),
            Some(PathBuf::from("/Users/ana/A & B/run/keep-alive"))
        );
        assert_eq!(
            strings(&text, "ProgramArguments").expect("arguments"),
            [
                "/Users/ana/Library/Application Support/Botloft/bin/botloftd",
                "serve",
                "--home",
                "/Users/ana/A & B",
                "--scheduled",
            ]
        );
    }

    #[test]
    fn without_starting_at_sign_in_it_does_not_run_at_load() {
        let text = plist_text("x", &task(false));
        assert!(text.contains("<key>RunAtLoad</key><false/>"));
        assert!(!text.contains("<key>RunAtLoad</key><true/>"));
    }

    #[test]
    fn each_data_folder_has_its_own_label() {
        assert_eq!(label("Botloft"), "io.github.httpsphl.botloft.daemon");
        assert_eq!(
            label("Botloft-1A2B3C4D"),
            "io.github.httpsphl.botloft.daemon-1a2b3c4d"
        );
    }
}
