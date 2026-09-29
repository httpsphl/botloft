//! How bot processes run. [`PipeRuntime`] starts them with stdin and stdout
//! in pipes (spec 7.4); [`fake::FakeRuntime`] implements the same contract
//! for tests, so the supervisor, chat and courier are tested without Claude.

pub mod claude;
pub mod fake;
mod pipe;

use std::ffi::OsString;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::pin::Pin;

use botloft_core::protocol::ClaudeAccount;
use bytes::Bytes;
use tokio::sync::mpsc;

pub use pipe::PipeRuntime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnSpec {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
    /// The complete environment. Nothing is inherited from the daemon.
    pub env: Vec<(OsString, OsString)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessEvent {
    /// Bytes from stdout, in order; lines may be split across chunks.
    Output(Bytes),
    /// Last event. The exit code, when the OS reported one.
    Exited(Option<u32>),
}

/// Handle to a running process. Dropping it kills the process tree.
pub trait ProcessControl: Send + Sync {
    /// Queues bytes for the process's stdin without blocking.
    fn write(&self, data: Bytes) -> io::Result<()>;
    /// Kills the process and everything it started.
    fn kill(&self) -> io::Result<()>;
}

pub struct Process {
    pub pid: Option<u32>,
    /// Output in order, then one [`ProcessEvent::Exited`].
    pub events: mpsc::Receiver<ProcessEvent>,
    pub control: Box<dyn ProcessControl>,
}

/// What `claude auth status` says: signed in or not, and to which account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthStatus {
    pub signed_in: bool,
    pub account: Option<ClaudeAccount>,
}

/// An [`AuthStatus`], computed off the async workers.
pub type AuthCheck = Pin<Box<dyn Future<Output = io::Result<AuthStatus>> + Send>>;

pub trait Runtime: Send + Sync + 'static {
    fn spawn(&self, spec: SpawnSpec) -> io::Result<Process>;
    /// Whether the Claude Code at `program` is signed in, and to which
    /// account (`claude auth status`).
    fn auth_status(&self, program: PathBuf) -> AuthCheck;
}

/// Capacity of the event channel between a process and the daemon. When it
/// is full, the reader thread waits, which in turn slows the process down.
pub(crate) const EVENT_BUFFER: usize = 64;
