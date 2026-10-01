//! A runtime that starts nothing (spec 1.6). Each spawn returns a
//! [`FakeProcess`] the test drives by hand: it emits stream-json events,
//! exits and records the lines the daemon wrote to its stdin.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use tokio::sync::{Notify, mpsc};

use serde_json::Value;

use botloft_core::protocol::ClaudeAccount;

use super::{
    AuthCheck, AuthStatus, EVENT_BUFFER, Process, ProcessControl, ProcessEvent, Runtime, SpawnSpec,
};

#[derive(Clone, Default)]
pub struct FakeRuntime {
    state: Arc<Mutex<State>>,
    spawned: Arc<Notify>,
}

struct State {
    processes: Vec<FakeProcess>,
    failures: usize,
    signed_in: bool,
    sign_in_checks: usize,
    hold: Option<SpawnHold>,
}

/// Holds the next spawn until the test lets it go.
struct SpawnHold {
    entered: std::sync::mpsc::Sender<()>,
    release: std::sync::mpsc::Receiver<()>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            processes: Vec::new(),
            failures: 0,
            signed_in: true,
            sign_in_checks: 0,
            hold: None,
        }
    }
}

#[derive(Clone)]
pub struct FakeProcess {
    pub spec: SpawnSpec,
    events: mpsc::Sender<ProcessEvent>,
    input: Arc<Mutex<Vec<u8>>>,
    wrote: Arc<Notify>,
    killed: Arc<AtomicBool>,
}

impl FakeRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    /// What `claude auth status` answers from now on (signed in by default).
    pub fn set_signed_in(&self, signed_in: bool) {
        lock(&self.state).signed_in = signed_in;
    }

    /// How many times the daemon asked whether Claude Code is signed in.
    pub fn sign_in_checks(&self) -> usize {
        lock(&self.state).sign_in_checks
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

    /// Makes the next spawn block, as a slow `CreateProcess` would. The
    /// first receiver hears when it begins; sending on the second ends it.
    pub fn hold_next_spawn(&self) -> (std::sync::mpsc::Receiver<()>, std::sync::mpsc::Sender<()>) {
        let (entered, began) = std::sync::mpsc::channel();
        let (release, held) = std::sync::mpsc::channel();
        lock(&self.state).hold = Some(SpawnHold {
            entered,
            release: held,
        });
        (began, release)
    }

    /// Makes the next spawn fail, as a missing executable would.
    pub fn fail_next_spawn(&self) {
        lock(&self.state).failures += 1;
    }
}

impl Runtime for FakeRuntime {
    fn auth_status(&self, _program: std::path::PathBuf) -> AuthCheck {
        let mut state = lock(&self.state);
        state.sign_in_checks += 1;
        let signed_in = state.signed_in;
        Box::pin(std::future::ready(Ok(AuthStatus {
            signed_in,
            account: signed_in.then(|| ClaudeAccount {
                email: Some("owner@example.com".into()),
                plan: Some("max".into()),
                organization: None,
            }),
        })))
    }

    fn spawn(&self, spec: SpawnSpec) -> io::Result<Process> {
        let hold = lock(&self.state).hold.take();
        if let Some(hold) = hold {
            let _ = hold.entered.send(());
            let _ = hold.release.recv();
        }
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
            wrote: Arc::default(),
            killed: Arc::default(),
        };
        let control = FakeControl {
            events,
            input: Arc::clone(&process.input),
            wrote: Arc::clone(&process.wrote),
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

    /// Writes one stream-json event to stdout, as Claude Code would.
    pub async fn emit(&self, event: Value) {
        let mut line = event.to_string().into_bytes();
        line.push(b'\n');
        self.output(&line).await;
    }

    pub async fn exit(&self, code: u32) {
        let _ = self.events.send(ProcessEvent::Exited(Some(code))).await;
    }

    pub fn input(&self) -> Vec<u8> {
        lock(&self.input).clone()
    }

    /// Every message written to stdin, parsed as JSON. What the daemon
    /// asks the process itself (spec 9.2) is in [`Self::control_requests`].
    pub fn input_lines(&self) -> Vec<Value> {
        self.stdin_lines()
            .into_iter()
            .filter(|line| line["type"] != "control_request")
            .collect()
    }

    /// Every control request written to stdin, by its `subtype`.
    pub fn control_requests(&self) -> Vec<String> {
        self.stdin_lines()
            .iter()
            .filter(|line| line["type"] == "control_request")
            .map(|line| {
                line["request"]["subtype"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect()
    }

    /// Answers the last control request of `subtype`, as Claude Code would.
    pub async fn answer_control(&self, subtype: &str, response: Value) {
        let id = self
            .stdin_lines()
            .iter()
            .rev()
            .find(|line| line["type"] == "control_request" && line["request"]["subtype"] == subtype)
            .map(|line| line["request_id"].clone())
            .unwrap_or_else(|| panic!("no {subtype} request on stdin"));
        self.emit(serde_json::json!({
            "type": "control_response",
            "response": { "subtype": "success", "request_id": id, "response": response },
        }))
        .await;
    }

    fn stdin_lines(&self) -> Vec<Value> {
        self.input()
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line).expect("stdin line is JSON"))
            .collect()
    }

    /// Waits up to 5 s until at least `n` lines were written to stdin.
    pub async fn wait_lines(&self, n: usize) -> Vec<Value> {
        let wait = async {
            loop {
                let notified = self.wrote.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                let lines = self.input_lines();
                if lines.len() >= n {
                    return lines;
                }
                notified.await;
            }
        };
        tokio::time::timeout(Duration::from_secs(5), wait)
            .await
            .unwrap_or_else(|_| panic!("expected {n} lines on stdin, got {:?}", self.input_lines()))
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
    wrote: Arc<Notify>,
    killed: Arc<AtomicBool>,
}

impl ProcessControl for FakeControl {
    fn write(&self, data: Bytes) -> io::Result<()> {
        if self.killed.load(Ordering::SeqCst) {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "killed"));
        }
        lock(&self.input).extend_from_slice(&data);
        self.wrote.notify_waiters();
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
