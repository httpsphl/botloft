//! Pseudo terminal runtime. ConPTY has no handle to poll, so each process
//! gets three threads (spec 8): one reads output, one writes input and one
//! waits for the exit. On Windows the process joins its own Job Object.

use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex, mpsc as std_mpsc};
use std::thread;

use bytes::Bytes;
use portable_pty::{ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};
use tokio::sync::mpsc;

use super::{EVENT_BUFFER, Process, ProcessControl, ProcessEvent, Runtime, SpawnSpec, TermSize};
use crate::platform::ProcessJob;

const READ_CHUNK: usize = 32 * 1024;

pub struct PtyRuntime;

impl Runtime for PtyRuntime {
    fn spawn(&self, spec: SpawnSpec) -> io::Result<Process> {
        let pair = native_pty_system()
            .openpty(pty_size(spec.size))
            .map_err(io::Error::other)?;
        let mut command = CommandBuilder::new(&spec.program);
        command.args(&spec.args);
        command.cwd(&spec.cwd);
        command.env_clear();
        for (name, value) in &spec.env {
            command.env(name, value);
        }

        let job = ProcessJob::new()?;
        let mut child = pair
            .slave
            .spawn_command(command)
            .map_err(io::Error::other)?;
        // The child holds its own end; keeping ours would stop EOF from arriving.
        drop(pair.slave);
        let mut killer = child.clone_killer();
        #[cfg(windows)]
        if let Some(handle) = child.as_raw_handle()
            && let Err(err) = job.assign(handle)
        {
            let _ = killer.kill();
            return Err(err);
        }

        let pid = child.process_id();
        let reader = pair.master.try_clone_reader().map_err(io::Error::other)?;
        let writer = pair.master.take_writer().map_err(io::Error::other)?;
        let master = Arc::new(Mutex::new(Some(pair.master)));
        let (events, events_rx) = mpsc::channel(EVENT_BUFFER);
        let (exit_tx, exit_rx) = std_mpsc::channel();

        let waiter_master = Arc::clone(&master);
        thread::Builder::new()
            .name("bot-wait".into())
            .spawn(move || {
                let code = child.wait().ok().map(|status| status.exit_code());
                let _ = exit_tx.send(code);
                // ConPTY keeps the output pipe open until the pseudo console
                // closes; dropping the master lets the reader see EOF.
                lock(&waiter_master).take();
            })?;
        thread::Builder::new()
            .name("bot-read".into())
            .spawn(move || read_output(reader, &events, &exit_rx))?;
        let (input, input_rx) = std_mpsc::channel::<Bytes>();
        thread::Builder::new()
            .name("bot-write".into())
            .spawn(move || write_input(writer, &input_rx))?;

        Ok(Process {
            pid,
            events: events_rx,
            control: Box::new(PtyControl {
                input: Mutex::new(input),
                master,
                killer: Mutex::new(killer),
                job,
            }),
        })
    }
}

fn read_output(
    mut reader: Box<dyn Read + Send>,
    events: &mpsc::Sender<ProcessEvent>,
    exit: &std_mpsc::Receiver<Option<u32>>,
) {
    let mut buf = vec![0u8; READ_CHUNK];
    loop {
        match reader.read(&mut buf) {
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

fn write_input(mut writer: Box<dyn Write + Send>, input: &std_mpsc::Receiver<Bytes>) {
    while let Ok(data) = input.recv() {
        if writer
            .write_all(&data)
            .and_then(|()| writer.flush())
            .is_err()
        {
            return;
        }
    }
}

struct PtyControl {
    input: Mutex<std_mpsc::Sender<Bytes>>,
    master: Arc<Mutex<Option<Box<dyn MasterPty + Send>>>>,
    killer: Mutex<Box<dyn ChildKiller + Send + Sync>>,
    job: ProcessJob,
}

impl ProcessControl for PtyControl {
    fn write(&self, data: Bytes) -> io::Result<()> {
        lock(&self.input)
            .send(data)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "the process has exited"))
    }

    fn resize(&self, size: TermSize) -> io::Result<()> {
        match lock(&self.master).as_ref() {
            Some(master) => master.resize(pty_size(size)).map_err(io::Error::other),
            None => Ok(()),
        }
    }

    fn kill(&self) -> io::Result<()> {
        let job = self.job.terminate();
        let child = lock(&self.killer).kill();
        job.or(child)
    }
}

fn pty_size(size: TermSize) -> PtySize {
    PtySize {
        rows: size.rows.max(1),
        cols: size.cols.max(1),
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
