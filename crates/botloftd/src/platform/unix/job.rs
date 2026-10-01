//! A process group per bot process, in place of the Job Object (spec 14.1):
//! the bot leads a group of its own, everything it starts joins it, and
//! stopping the bot kills the whole group.

use std::io;
use std::os::unix::process::CommandExt as _;
use std::process::{Child, Command};
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct ProcessJob {
    /// The group's id, which is its leader's pid; `None` until a process
    /// joins and after the group is gone.
    group: Mutex<Option<libc::pid_t>>,
}

impl ProcessJob {
    pub fn new() -> io::Result<Self> {
        Ok(Self::default())
    }

    /// Makes `command` start a process group of its own.
    pub fn prepare(command: &mut Command) {
        command.process_group(0);
    }

    /// Takes the group that `child`, started from a prepared command, leads.
    pub fn assign(&self, child: &Child) -> io::Result<()> {
        let pid = libc::pid_t::try_from(child.id()).map_err(io::Error::other)?;
        *self.lock() = Some(pid);
        Ok(())
    }

    /// Kills every process in the group.
    pub fn terminate(&self) -> io::Result<()> {
        let mut group = self.lock();
        let Some(id) = *group else {
            return Ok(());
        };
        // SAFETY: `kill` takes no pointers; a negative pid names the group.
        if unsafe { libc::kill(-id, libc::SIGKILL) } == 0 {
            return Ok(());
        }
        let err = io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ESRCH) {
            // Nobody is left. Forgetting the id keeps a later call from
            // reaching a new group that happens to get the same number.
            *group = None;
            return Ok(());
        }
        Err(err)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<libc::pid_t>> {
        self.group
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Like a closed Job Object handle: whatever is left of the group dies.
impl Drop for ProcessJob {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    use super::*;

    fn alive(pid: libc::pid_t) -> bool {
        // SAFETY: signal 0 only checks that the process exists.
        unsafe { libc::kill(pid, 0) == 0 }
    }

    #[test]
    fn terminating_kills_what_the_process_started() {
        let job = ProcessJob::new().expect("job");
        let mut command = Command::new("sh");
        command
            .args(["-c", "sleep 30 & echo $!; wait"])
            .stdout(Stdio::piped());
        ProcessJob::prepare(&mut command);
        let mut child = command.spawn().expect("spawn");
        job.assign(&child).expect("assign");
        let mut line = String::new();
        BufReader::new(child.stdout.take().expect("stdout"))
            .read_line(&mut line)
            .expect("read");
        let started: libc::pid_t = line.trim().parse().expect("pid");
        assert!(alive(started));

        job.terminate().expect("terminate");
        child.wait().expect("wait");
        let deadline = Instant::now() + Duration::from_secs(5);
        while alive(started) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!alive(started), "the process the child started is gone");
        job.terminate().expect("terminating again is fine");
    }
}
