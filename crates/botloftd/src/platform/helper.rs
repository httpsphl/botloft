//! Small programs the daemon runs and reads the answer of, such as `git`
//! and `gh` for the Overview (spec 33.5): no window, no input, a time limit.

use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::ProcessJob;

/// Runs `program` in `cwd` and waits for it, up to `limit`; past it, the
/// program is killed and the answer is a `TimedOut` error. A program that is
/// not installed is a `NotFound` error.
pub fn run_helper(
    program: &str,
    args: &[&str],
    cwd: &Path,
    env: &[(&str, &str)],
    limit: Duration,
) -> io::Result<Output> {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .envs(env.iter().copied())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    ProcessJob::prepare(&mut command);
    let mut child = command.spawn()?;
    // Read as it writes: a full pipe would stop the program.
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() > limit {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("{program} took longer than {} ms", limit.as_millis()),
            ));
        }
        thread::sleep(Duration::from_millis(10));
    };
    Ok(Output {
        status,
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
    })
}

fn drain(pipe: Option<impl Read + Send + 'static>) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        bytes
    })
}
