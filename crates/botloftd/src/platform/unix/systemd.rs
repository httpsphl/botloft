//! The daemon as a `systemd --user` service on Linux (spec 14.1). Enabled,
//! the owner's user manager starts it when they sign in; while it runs,
//! systemd brings it back after a crash. Stopping the service ends every
//! process in its control group, the bots and their browsers too.

use std::io;
use std::path::PathBuf;
use std::process::Output;

use super::write_atomically;
use crate::platform::{TaskDefinition, TaskInfo, TaskState, Triggers};

pub const TASK_KIND: &str = "systemd user service";

fn unit(name: &str) -> String {
    format!("{}.service", name.to_lowercase())
}

fn unit_path(name: &str) -> io::Result<PathBuf> {
    let config = dirs::config_dir()
        .ok_or_else(|| io::Error::other("cannot find the configuration folder"))?;
    Ok(config.join("systemd").join("user").join(unit(name)))
}

fn systemctl(args: &[&str]) -> io::Result<Output> {
    let mut all = vec!["--user"];
    all.extend_from_slice(args);
    super::manager("systemctl", &all)
}

/// Runs `systemctl --user` and fails with what it said.
fn run(args: &[&str]) -> io::Result<()> {
    let output = systemctl(args)?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "`systemctl --user {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    )))
}

/// The first word `systemctl --user <query> <unit>` prints, whatever its
/// exit code (`inactive` and `disabled` come with a failure code).
fn query(query: &str, unit: &str) -> Option<String> {
    let output = systemctl(&[query, unit]).ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.split_whitespace().next().map(str::to_owned)
}

pub fn register_task(name: &str, task: &TaskDefinition) -> io::Result<()> {
    let path = unit_path(name)?;
    write_atomically(&path, unit_text(task).as_bytes())?;
    run(&["daemon-reload"])?;
    let verb = if task.triggers.logon {
        "enable"
    } else {
        "disable"
    };
    run(&[verb, &unit(name)])
}

pub fn find_task(name: &str) -> io::Result<Option<TaskInfo>> {
    let path = unit_path(name)?;
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    };
    let unit = unit(name);
    let state = match query("is-active", &unit).as_deref() {
        Some("active" | "activating" | "reloading" | "deactivating") => TaskState::Running,
        Some("inactive" | "failed") => TaskState::Ready,
        _ => TaskState::Unknown,
    };
    let logon = query("is-enabled", &unit).as_deref() == Some("enabled");
    Ok(Some(TaskInfo {
        state,
        command: exec_line(&text),
        triggers: Triggers {
            logon,
            watchdog: true,
        },
    }))
}

pub fn run_task(name: &str) -> io::Result<()> {
    run(&["start", &unit(name)])
}

pub fn stop_task(name: &str) -> io::Result<()> {
    run(&["stop", &unit(name)])
}

pub fn delete_task(name: &str) -> io::Result<bool> {
    let path = unit_path(name)?;
    if !path.exists() {
        return Ok(false);
    }
    let unit = unit(name);
    // Errors here only mean there was nothing to stop or disable.
    let _ = systemctl(&["stop", &unit]);
    let _ = systemctl(&["disable", &unit]);
    std::fs::remove_file(&path)?;
    let _ = systemctl(&["daemon-reload"]);
    let _ = systemctl(&["reset-failed", &unit]);
    Ok(true)
}

/// The unit. `Restart=on-failure` brings the daemon back after a crash but
/// not after a stop asked for (logout, `service stop`), which ends with a
/// clean exit. The default `KillMode=control-group` is spelled out: it is
/// what ends the bots with the daemon.
fn unit_text(task: &TaskDefinition) -> String {
    let exec = std::iter::once(task.program.to_string_lossy().into_owned())
        .chain(task.arguments.iter().cloned())
        .map(|arg| quote(&arg))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "# Written by Botloft (spec 14.1). `botloftd service uninstall` removes it.\n\
         [Unit]\n\
         Description={description}\n\
         \n\
         [Service]\n\
         Type=exec\n\
         ExecStart={exec}\n\
         WorkingDirectory={dir}\n\
         Restart=on-failure\n\
         RestartSec=5\n\
         KillMode=control-group\n\
         TimeoutStopSec=20\n\
         \n\
         [Install]\n\
         WantedBy=default.target\n",
        description = specifiers(&task.description),
        dir = specifiers(&task.working_dir.to_string_lossy()),
    )
}

/// `%` starts a specifier in a unit file.
fn specifiers(text: &str) -> String {
    text.replace('%', "%%")
}

/// One word of `ExecStart`, in double quotes: `\` and `"` escaped, `%` and
/// `$` doubled so systemd takes them as they are.
fn quote(arg: &str) -> String {
    let mut out = String::from("\"");
    for c in arg.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '%' => out.push_str("%%"),
            '$' => out.push_str("$$"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn exec_line(unit: &str) -> Option<String> {
    unit.lines()
        .find_map(|line| line.strip_prefix("ExecStart="))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn task() -> TaskDefinition {
        TaskDefinition {
            description: "Starts Botloft. Data folder: /home/ana/100% bots".into(),
            program: PathBuf::from("/home/ana/100% bots/bin/botloftd"),
            arguments: ["serve", "--home", "/home/ana/100% bots", "--scheduled"]
                .map(String::from)
                .to_vec(),
            working_dir: PathBuf::from("/home/ana/100% bots"),
            keep_alive: PathBuf::from("/home/ana/100% bots/run/keep-alive"),
            triggers: Triggers {
                logon: true,
                watchdog: true,
            },
        }
    }

    #[test]
    fn the_unit_runs_the_daemon_and_ends_its_processes_with_it() {
        let text = unit_text(&task());
        assert!(text.contains(
            r#"ExecStart="/home/ana/100%% bots/bin/botloftd" "serve" "--home" "/home/ana/100%% bots" "--scheduled""#
        ));
        assert!(text.contains("WorkingDirectory=/home/ana/100%% bots\n"));
        assert!(text.contains("Description=Starts Botloft. Data folder: /home/ana/100%% bots\n"));
        assert!(text.contains("Restart=on-failure\n"));
        assert!(text.contains("KillMode=control-group\n"));
        assert!(text.contains("WantedBy=default.target\n"));
        assert_eq!(
            exec_line(&text).as_deref(),
            Some(
                r#""/home/ana/100%% bots/bin/botloftd" "serve" "--home" "/home/ana/100%% bots" "--scheduled""#
            )
        );
    }

    #[test]
    fn quotes_and_dollars_stay_as_they_are() {
        assert_eq!(quote(r#"a "b" \c $HOME"#), r#""a \"b\" \\c $$HOME""#);
    }

    #[test]
    fn each_data_folder_has_its_own_unit() {
        assert_eq!(unit("Botloft"), "botloft.service");
        assert_eq!(unit("Botloft-1a2b3c4d"), "botloft-1a2b3c4d.service");
    }
}
