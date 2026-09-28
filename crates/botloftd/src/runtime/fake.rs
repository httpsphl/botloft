//! A runtime that starts nothing (spec 1.6). Each spawn returns a
//! [`FakeProcess`] the test drives by hand: it emits output, exits and
//! records what the daemon wrote to it.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use tokio::sync::{Notify, mpsc};

use super::{EVENT_BUFFER, Process, ProcessControl, ProcessEvent, Runtime, SpawnSpec, TermSize};

#[derive(Clone, Default)]
pub struct FakeRuntime {
    state: Arc<Mutex<State>>,
    spawned: Arc<Notify>,
}

#[derive(Default)]
struct State {
    processes: Vec<FakeProcess>,
    failures: usize,
}

#[derive(Clone)]
pub struct FakeProcess {
    pub spec: SpawnSpec,
    events: mpsc::Sender<ProcessEvent>,
    input: Arc<Mutex<Vec<u8>>>,
    sizes: Arc<Mutex<Vec<TermSize>>>,
    killed: Arc<AtomicBool>,
}

impl FakeRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    /// Every process spawned so far, oldest first.
    pub fn processes(&self) -> Vec<FakeProcess> {
        lock(&self.state).processes.clone()
    }

    /// The `n`-th spawned process (1-based), waiting up to 5 s for it.
    pub async fn process(&self, n: usize) -> FakeProcess {
        let wait = async {
            loop {
                // Registered before the check, so a spawn in between still wakes us.
                let notified = self.spawned.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                if let Some(process) = lock(&self.state).processes.get(n - 1) {
                    return process.clone();
                }
                notified.await;
            }
        };
        tokio::time::timeout(Duration::from_secs(5), wait)
            .await
            .unwrap_or_else(|_| panic!("process {n} was never spawned"))
    }

    /// Makes the next spawn fail, as a missing executable would.
    pub fn fail_next_spawn(&self) {
        lock(&self.state).failures += 1;
    }
}

impl Runtime for FakeRuntime {
    fn spawn(&self, spec: SpawnSpec) -> io::Result<Process> {
        let mut state = lock(&self.state);
        if state.failures > 0 {
            state.failures -= 1;
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "fake spawn failure",
            ));
        }
        let (events, events_rx) = mpsc::channel(EVENT_BUFFER);
        let process = FakeProcess {
            spec,
            events: events.clone(),
            input: Arc::default(),
            sizes: Arc::default(),
            killed: Arc::default(),
        };
        let control = FakeControl {
            events,
            input: Arc::clone(&process.input),
            sizes: Arc::clone(&process.sizes),
            killed: Arc::clone(&process.killed),
        };
        state.processes.push(process);
        drop(state);
        self.spawned.notify_waiters();
        Ok(Process {
            pid: None,
            events: events_rx,
            control: Box::new(control),
        })
    }
}

impl FakeProcess {
    pub async fn output(&self, data: &[u8]) {
        let _ = self
            .events
            .send(ProcessEvent::Output(Bytes::copy_from_slice(data)))
            .await;
    }

    pub async fn exit(&self, code: u32) {
        let _ = self.events.send(ProcessEvent::Exited(Some(code))).await;
    }

    pub fn input(&self) -> Vec<u8> {
        lock(&self.input).clone()
    }

    pub fn sizes(&self) -> Vec<TermSize> {
        lock(&self.sizes).clone()
    }

    pub fn killed(&self) -> bool {
        self.killed.load(Ordering::SeqCst)
    }

    pub fn args(&self) -> Vec<String> {
        self.spec
            .args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    pub fn env(&self, name: &str) -> Option<String> {
        self.spec
            .env
            .iter()
            .find(|(key, _)| key.to_string_lossy().eq_ignore_ascii_case(name))
            .map(|(_, value)| value.to_string_lossy().into_owned())
    }
}

struct FakeControl {
    events: mpsc::Sender<ProcessEvent>,
    input: Arc<Mutex<Vec<u8>>>,
    sizes: Arc<Mutex<Vec<TermSize>>>,
    killed: Arc<AtomicBool>,
}

impl ProcessControl for FakeControl {
    fn write(&self, data: Bytes) -> io::Result<()> {
        lock(&self.input).extend_from_slice(&data);
        Ok(())
    }

    fn resize(&self, size: TermSize) -> io::Result<()> {
        lock(&self.sizes).push(size);
        Ok(())
    }

    /// Dies like a real process would: the exit event follows.
    fn kill(&self) -> io::Result<()> {
        if !self.killed.swap(true, Ordering::SeqCst) {
            let _ = self.events.try_send(ProcessEvent::Exited(None));
        }
        Ok(())
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
