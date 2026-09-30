//! Where the folder of a deleted bot or crew goes when the owner asks
//! (spec 7.6): the Recycle Bin, behind a trait so tests move no folder
//! into a real one.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use botloft_core::protocol::FolderRecycled;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::platform;
use crate::state::Event;

/// How many times a folder that cannot move yet is tried, and how long
/// between tries: a process that was just killed still holds its files.
const TRIES: u32 = 20;
const RETRY_EVERY: Duration = Duration::from_millis(500);

pub trait Trash: Send + Sync {
    /// Moves the folder away, where the owner can get it back, or fails
    /// and leaves it as it is. `Unsupported` means it can never go there.
    fn put(&self, folder: &Path) -> io::Result<()>;
}

/// The Windows Recycle Bin.
pub struct RecycleBin;

impl Trash for RecycleBin {
    fn put(&self, folder: &Path) -> io::Result<()> {
        platform::recycle(folder)
    }
}

/// Moves `folder` to the trash off the async workers, trying again while
/// something still holds it, and tells the apps how it ended
/// (`folder.recycled`).
pub(crate) fn move_away(trash: Arc<dyn Trash>, events: broadcast::Sender<Event>, folder: PathBuf) {
    let work = move || {
        let error = put_with_retries(trash.as_ref(), &folder)
            .err()
            .map(|err| err.to_string());
        match &error {
            None => info!("a deleted bot's folder went to the Recycle Bin"),
            Some(error) => warn!("a folder did not go to the Recycle Bin: {error}"),
        }
        // No receivers just means no app is connected.
        let _ = events.send(Event::FolderRecycled(FolderRecycled {
            path: folder.to_string_lossy().into_owned(),
            error,
        }));
    };
    match tokio::runtime::Handle::try_current() {
        Ok(runtime) => drop(runtime.spawn_blocking(work)),
        Err(_) => work(),
    }
}

fn put_with_retries(trash: &dyn Trash, folder: &Path) -> io::Result<()> {
    let mut tries = 0;
    loop {
        tries += 1;
        match trash.put(folder) {
            Ok(()) => return Ok(()),
            // Already gone: there is nothing left to move.
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(err) if err.kind() == io::ErrorKind::Unsupported || tries >= TRIES => {
                return Err(err);
            }
            Err(_) => std::thread::sleep(RETRY_EVERY),
        }
    }
}

/// A trash for tests: folders move into `bin`, and the next tries can be
/// made to fail.
#[derive(Clone)]
pub struct FakeTrash {
    bin: PathBuf,
    state: Arc<Mutex<FakeState>>,
}

#[derive(Default)]
struct FakeState {
    moved: Vec<PathBuf>,
    /// Errors the next tries answer with, first to last.
    failures: Vec<io::ErrorKind>,
    tries: usize,
}

impl FakeTrash {
    pub fn new(bin: PathBuf) -> Self {
        Self {
            bin,
            state: Arc::default(),
        }
    }

    /// The folders that were moved, in order, as they were named.
    pub fn moved(&self) -> Vec<PathBuf> {
        self.lock().moved.clone()
    }

    /// How many times a move was tried.
    pub fn tries(&self) -> usize {
        self.lock().tries
    }

    /// The next tries fail with these, in order, before one may work.
    pub fn fail_next(&self, failures: &[io::ErrorKind]) {
        self.lock().failures = failures.to_vec();
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, FakeState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Trash for FakeTrash {
    fn put(&self, folder: &Path) -> io::Result<()> {
        let mut state = self.lock();
        state.tries += 1;
        if !state.failures.is_empty() {
            return Err(io::Error::new(state.failures.remove(0), "fake failure"));
        }
        if !folder.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "no such folder"));
        }
        std::fs::create_dir_all(&self.bin)?;
        std::fs::rename(folder, self.bin.join(state.moved.len().to_string()))?;
        state.moved.push(folder.to_owned());
        Ok(())
    }
}
