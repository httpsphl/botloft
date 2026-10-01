//! Bot and browser groups a crashed daemon leaves behind (spec 14.1). On
//! macOS, launchd ends only the daemon's own process group, and a daemon
//! started by hand has nobody to end them. Each group is written down when
//! it starts, with its arguments; the next daemon of the same data folder
//! ends the ones still running all of those arguments, so a number reused
//! by another program is never touched.

use std::fs::OpenOptions;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

const FILE_NAME: &str = "groups";

/// Where this daemon writes its groups down, once `track_groups` ran.
static FILE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    group: libc::pid_t,
    args: Vec<String>,
}

/// Ends the groups the last daemon of this data folder left running, then
/// writes this daemon's groups down in `dir`.
pub fn track_groups(dir: &Path) {
    let file = dir.join(FILE_NAME);
    let ended = sweep(&file);
    if ended > 0 {
        info!(ended, "ended processes a previous run left behind");
    }
    if let Err(err) = std::fs::create_dir_all(dir) {
        debug!("cannot create {}: {err}", dir.display());
    }
    let _ = FILE.set(file);
}

/// Writes down the group `command` started.
pub(super) fn remember(group: libc::pid_t, command: &Command) {
    if let Some(file) = FILE.get() {
        write_down(file, group, command);
    }
}

fn write_down(file: &Path, group: libc::pid_t, command: &Command) {
    let entry = Entry {
        group,
        args: command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect(),
    };
    let written = serde_json::to_string(&entry)
        .map_err(std::io::Error::other)
        .and_then(|line| {
            let mut out = OpenOptions::new().create(true).append(true).open(file)?;
            super::restrict_to_current_user(file)?;
            out.write_all(format!("{line}\n").as_bytes())
        });
    if let Err(err) = written {
        debug!("cannot write a process group down: {err}");
    }
}

/// Ends every group in `file` still running its arguments, and empties it.
fn sweep(file: &Path) -> usize {
    let Ok(opened) = std::fs::File::open(file) else {
        return 0;
    };
    let mut ended = 0;
    for line in BufReader::new(opened).lines().map_while(Result::ok) {
        let Ok(entry) = serde_json::from_str::<Entry>(&line) else {
            continue;
        };
        // SAFETY: `kill` takes no pointers; a negative pid names the group.
        if still_running(&entry) && unsafe { libc::kill(-entry.group, libc::SIGKILL) } == 0 {
            ended += 1;
        }
    }
    let _ = std::fs::remove_file(file);
    ended
}

/// Whether the group's leader still runs with every argument it had.
fn still_running(entry: &Entry) -> bool {
    if entry.args.is_empty() || entry.group <= 1 {
        return false;
    }
    let Ok(output) = Command::new("ps")
        .args(["-ww", "-o", "command=", "-p", &entry.group.to_string()])
        .output()
    else {
        return false;
    };
    let line = String::from_utf8_lossy(&output.stdout);
    let line = line.trim();
    !line.is_empty() && entry.args.iter().all(|arg| line.contains(arg.as_str()))
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::CommandExt as _;
    use std::time::{Duration, Instant};

    use super::*;

    fn alive(pid: libc::pid_t) -> bool {
        // SAFETY: signal 0 only checks that the process exists.
        unsafe { libc::kill(pid, 0) == 0 }
    }

    #[test]
    fn a_group_left_running_is_ended_and_others_are_not() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join(FILE_NAME);
        let marker = format!("left-behind-{}", std::process::id());
        let mut command = Command::new("sh");
        command
            .args(["-c", &format!("sleep 300; : {marker}")])
            .process_group(0);
        let mut child = command.spawn().expect("spawn");
        let group = libc::pid_t::try_from(child.id()).expect("pid");
        write_down(&file, group, &command);
        // This test's own process, with arguments it never had.
        let own = libc::pid_t::try_from(std::process::id()).expect("pid");
        let mut other = Command::new("sh");
        other.args(["-c", "not this one"]);
        write_down(&file, own, &other);

        assert_eq!(sweep(&file), 1);
        child.wait().expect("wait");
        let deadline = Instant::now() + Duration::from_secs(5);
        while alive(group) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!alive(group), "the group left behind is gone");
        assert!(!file.exists(), "the list starts over");
        assert_eq!(sweep(&file), 0);
    }
}
