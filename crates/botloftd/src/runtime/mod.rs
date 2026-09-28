//! How bot processes run. [`PtyRuntime`] starts them in a pseudo terminal
//! (ConPTY on Windows); [`fake::FakeRuntime`] implements the same contract
//! for tests, so the supervisor and terminal are tested without Claude.

pub mod claude;
pub mod fake;
mod pty;

use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use bytes::Bytes;
use tokio::sync::mpsc;

pub use pty::PtyRuntime;

/// Terminal size in character cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermSize {
    pub cols: u16,
    pub rows: u16,
}

impl Default for TermSize {
    /// Used until the app sends `terminal.resize` (spec 7.4).
    fn default() -> Self {
        Self {
            cols: 120,
            rows: 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnSpec {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
    /// The complete environment. Nothing is inherited from the daemon.
    pub env: Vec<(OsString, OsString)>,
    pub size: TermSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessEvent {
    Output(Bytes),
    /// Last event. The exit code, when the OS reported one.
    Exited(Option<u32>),
}

/// Handle to a running process. Dropping it kills the process tree.
pub trait ProcessControl: Send + Sync {
    /// Queues bytes for the process's input without blocking.
    fn write(&self, data: Bytes) -> io::Result<()>;
    fn resize(&self, size: TermSize) -> io::Result<()>;
    /// Kills the process and everything it started.
    fn kill(&self) -> io::Result<()>;
}

pub struct Process {
    pub pid: Option<u32>,
    /// Output in order, then one [`ProcessEvent::Exited`].
    pub events: mpsc::Receiver<ProcessEvent>,
    pub control: Box<dyn ProcessControl>,
}

pub trait Runtime: Send + Sync + 'static {
    fn spawn(&self, spec: SpawnSpec) -> io::Result<Process>;
}

/// Capacity of the event channel between a process and the daemon. When it
/// is full, the reader thread waits, which in turn slows the process down.
pub(crate) const EVENT_BUFFER: usize = 64;
