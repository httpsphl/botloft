//! Processes with stdin, stdout and stderr in pipes (spec 7.4). Each one
//! gets a thread to read stdout, one to write stdin (writing never blocks
//! the caller), one to drain stderr and one to wait for the exit. On
//! Windows the process joins its own Job Object and has no console.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex, mpsc as std_mpsc};
use std::thread;

use bytes::Bytes;
use tokio::sync::mpsc;
use tracing::debug;

use super::{EVENT_BUFFER, Process, ProcessControl, ProcessEvent, Runtime, SpawnSpec};
use crate::platform::ProcessJob;

const READ_CHUNK: usize = 64 * 1024;
/// Longest stderr line kept for the debug log.
const STDERR_LINE_MAX: usize = 300;

pub struct PipeRuntime;

impl Runtime for PipeRuntime {
    fn spawn(&self, spec: SpawnSpec) -> io::Result<Process> {
        let mut command = Command::new(&spec.program);
        command
            .args(&spec.args)
            .current_dir(&spec.cwd)
            .env_clear()
            .envs(spec.env.iter().map(|(name, value)| (name, value)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt as _;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let job = Arc::new(ProcessJob::new()?);
        let mut child = command.spawn()?;
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle as _;
            if let Err(err) = job.assign(child.as_raw_handle()) {
                let _ = child.kill();
                return Err(err);
            }
        }
        let pid = Some(child.id());
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("no stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| io::Error::other("no stderr"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("no stdin"))?;
        let child = Arc::new(Mutex::new(child));

        let (events, events_rx) = mpsc::channel(EVENT_BUFFER);
        let (exit_tx, exit_rx) = std_mpsc::channel();
        let waiter_child = Arc::clone(&child);
        let waiter_job = Arc::clone(&job);
        thread::Builder::new()
            .name("bot-wait".into())
            .spawn(move || {
                let code = wait(&waiter_child);
                let _ = exit_tx.send(code);
                // Something the bot started may still hold stdout open;
                // ending the job lets the reader see EOF.
                let _ = waiter_job.terminate();
            })?;
        thread::Builder::new()
            .name("bot-read".into())
            .spawn(move || read_output(stdout, &events, &exit_rx))?;
        thread::Builder::new()
            .name("bot-stderr".into())
            .spawn(move || drain_stderr(stderr))?;
        let (input, input_rx) = std_mpsc::channel::<Bytes>();
        thread::Builder::new()
            .name("bot-write".into())
            .spawn(move || write_input(stdin, &input_rx))?;

        Ok(Process {
            pid,
            events: events_rx,
            control: Box::new(PipeControl {
                input: Mutex::new(Some(input)),
                child,
                job,
            }),
        })
    }
}

/// Polls instead of blocking in `wait`, so `kill` can take the lock.
fn wait(child: &Mutex<Child>) -> Option<u32> {
    loop {
        match lock(child).try_wait() {
            Ok(Some(status)) => return status.code().and_then(|code| u32::try_from(code).ok()),
            Ok(None) => {}
            Err(_) => return None,
        }
        thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn read_output(
    mut stdout: impl Read,
    events: &mpsc::Sender<ProcessEvent>,
    exit: &std_mpsc::Receiver<Option<u32>>,
) {
    let mut buf = vec![0u8; READ_CHUNK];
    loop {
        match stdout.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                let chunk = Bytes::copy_from_slice(&buf[..n]);
                if events.blocking_send(ProcessEvent::Output(chunk)).is_err() {
                    break;
                }
            }
        }
    }
    let code = exit.recv().ok().flatten();
    let _ = events.blocking_send(ProcessEvent::Exited(code));
}

/// Claude Code writes warnings here. Only their start goes to the debug
/// log: a line could quote the conversation.
fn drain_stderr(stderr: impl Read) {
    for line in BufReader::new(stderr).lines() {
        let Ok(line) = line else { break };
        let short: String = line.chars().take(STDERR_LINE_MAX).collect();
        debug!(target: "botloftd::bot_stderr", "{short}");
    }
}

fn write_input(mut stdin: ChildStdin, input: &std_mpsc::Receiver<Bytes>) {
    while let Ok(data) = input.recv() {
        if stdin.write_all(&data).and_then(|()| stdin.flush()).is_err() {
            return;
        }
    }
    // The sender was dropped: closing stdin ends the Claude Code session.
}

struct PipeControl {
    /// `None` after the process was killed.
    input: Mutex<Option<std_mpsc::Sender<Bytes>>>,
    child: Arc<Mutex<Child>>,
    job: Arc<ProcessJob>,
}

impl ProcessControl for PipeControl {
    fn write(&self, data: Bytes) -> io::Result<()> {
        let closed = || io::Error::new(io::ErrorKind::BrokenPipe, "the process has exited");
        lock(&self.input)
            .as_ref()
            .ok_or_else(closed)?
            .send(data)
            .map_err(|_| closed())
    }

    fn kill(&self) -> io::Result<()> {
        lock(&self.input).take();
        let job = self.job.terminate();
        let child = lock(&self.child).kill();
        job.or(child)
    }
}

impl Drop for PipeControl {
    fn drop(&mut self) {
        let _ = self.kill();
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
