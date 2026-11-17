//! Waits (spec 20.14): a bot waits for a file, the end of a program or a
//! text on a page, and Botloft looks for it by itself, without the model.
//! The bot wakes once: when it happened, or when the time ran out.

use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use botloft_core::ids::RoutineId;
use botloft_core::protocol::{Routine, Schedule, WaitEnd, WaitOn};
use tracing::warn;

use super::fire::{self, Trigger};
use super::wait_page;
use crate::service::bots;
use crate::state::Daemon;

/// How often the waits are looked at.
pub const EVERY: Duration = Duration::from_secs(5);
/// A page is read at most this often for each wait.
const PAGE_EVERY_MS: i64 = 60_000;

/// A page read: its text, or `None` when it could not be read.
pub type PageText<'a> = Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>>;

/// What a wait looks at. Tests give their own.
pub trait WaitProbe: Send + Sync {
    fn file_exists(&self, path: &Path) -> bool;
    /// The names of the running programs; `None` when they cannot be listed.
    fn programs(&self) -> Option<Vec<String>>;
    fn page(&self, url: &str) -> PageText<'_>;
}

/// The computer itself.
pub struct SystemProbe;

impl WaitProbe for SystemProbe {
    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn programs(&self) -> Option<Vec<String>> {
        crate::platform::running_programs()
    }

    fn page(&self, url: &str) -> PageText<'_> {
        let url = url.to_owned();
        Box::pin(async move { wait_page::fetch(&url).await })
    }
}

/// A program's name as compared: the file name only, lowercase, without
/// `.exe` (`C:\Tools\FFmpeg.EXE` and `ffmpeg` are the same).
pub(crate) fn program_key(name: &str) -> String {
    let file = name.rsplit(['/', '\\']).next().unwrap_or(name).trim();
    let lower = file.to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_owned()
}

/// Whether a program called `name` is in `running`. Linux cuts the names
/// `ps` gives at 15 characters, so a cut name counts too.
pub(crate) fn is_running(running: &[String], name: &str) -> bool {
    let wanted = program_key(name);
    running
        .iter()
        .map(|each| program_key(each))
        .any(|each| each == wanted || (each.chars().count() == 15 && wanted.starts_with(&each)))
}

/// Runs the waits until the daemon stops.
pub async fn run(daemon: Arc<Daemon>) {
    loop {
        check(&daemon).await;
        tokio::time::sleep(EVERY).await;
    }
}

/// One look at every open wait; each that happened or ran out wakes its
/// bot. Tests call it after setting a probe and moving the clock.
pub async fn check(daemon: &Daemon) {
    let waits = match daemon.store().wait_routines() {
        Ok(waits) => waits,
        Err(err) => {
            warn!("could not read the waits: {err}");
            return;
        }
    };
    if waits.is_empty() {
        return;
    }
    let probe = daemon.routines.probe();
    let wants_programs = waits
        .iter()
        .any(|routine| matches!(on(routine), Some(WaitOn::Program { .. })));
    let programs = if wants_programs {
        let probe = Arc::clone(&probe);
        tokio::task::spawn_blocking(move || probe.programs())
            .await
            .ok()
            .flatten()
    } else {
        None
    };
    for routine in waits {
        let (Some(on), Schedule::Wait { until, .. }) = (on(&routine), &routine.schedule) else {
            continue;
        };
        let until = *until;
        let happened = match on {
            WaitOn::File { path } => probe.file_exists(Path::new(path)),
            // Unknown when the list could not be read: not yet.
            WaitOn::Program { name } => programs
                .as_ref()
                .is_some_and(|running| !is_running(running, name)),
            WaitOn::Page { url, text } => {
                let now = daemon.clock.now_ms();
                if daemon.routines.page_due(&routine.id, now, PAGE_EVERY_MS) {
                    probe
                        .page(url)
                        .await
                        .is_some_and(|page| wait_page::shows(&page, text))
                } else {
                    false
                }
            }
        };
        let now = daemon.clock.now_ms();
        let ended = if happened {
            WaitEnd::Happened
        } else if now >= until {
            WaitEnd::TimedOut
        } else {
            continue;
        };
        ended_with(daemon, routine, ended, now);
    }
}

fn on(routine: &Routine) -> Option<&WaitOn> {
    match &routine.schedule {
        Schedule::Wait { on, .. } => Some(on),
        _ => None,
    }
}

/// The wait wakes its bot once, then goes, like a bot's own return (spec
/// 20.12). A paused bot or crew only delays the message.
fn ended_with(daemon: &Daemon, mut routine: Routine, ended: WaitEnd, now: i64) {
    daemon.routines.forget_page(&routine.id);
    let store = daemon.store();
    match bots::active(&store, &routine.bot_id) {
        Ok((crew, bot)) => {
            let fired = fire::attempt(
                daemon,
                &store,
                &routine,
                &crew,
                &bot,
                now,
                Trigger::Wait(ended),
            );
            if let Err(err) = fired {
                warn!(routine = %routine.id, "a wait could not wake its bot: {err}");
            }
        }
        Err(err) => warn!(routine = %routine.id, "a wait's bot is gone: {err}"),
    }
    routine.archived_at = Some(now);
    routine.next_run_at = None;
    routine.updated_at = now;
    drop(store);
    fire::save(daemon, &routine);
}

/// When each wait on a page last read it.
#[derive(Default)]
pub(super) struct PageTimes(HashMap<RoutineId, i64>);

impl PageTimes {
    /// Whether the page is due a read now; if so, the read counts from now.
    pub(super) fn due(&mut self, id: &RoutineId, now: i64, every: i64) -> bool {
        match self.0.get(id) {
            Some(last) if now - last < every => false,
            _ => {
                self.0.insert(id.clone(), now);
                true
            }
        }
    }

    pub(super) fn forget(&mut self, id: &RoutineId) {
        self.0.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::{PageTimes, is_running, program_key};
    use botloft_core::ids::RoutineId;

    #[test]
    fn programs_compare_by_file_name_without_exe() {
        assert_eq!(program_key(r"C:\Tools\FFmpeg.EXE"), "ffmpeg");
        assert_eq!(program_key("/usr/bin/ffmpeg"), "ffmpeg");
        let running = vec!["explorer.exe".to_owned(), "ffmpeg.exe".to_owned()];
        assert!(is_running(&running, "FFMPEG"));
        assert!(is_running(&running, "ffmpeg.exe"));
        assert!(!is_running(&running, "ffprobe"));
        // Linux cuts what ps shows at 15 characters.
        let cut = vec!["libreoffice-cal".to_owned()];
        assert!(is_running(&cut, "libreoffice-calc"));
    }

    #[test]
    fn the_system_lists_this_very_program() {
        use super::{SystemProbe, WaitProbe};
        let me = std::env::current_exe().expect("exe");
        let name = me.file_name().expect("name").to_string_lossy().into_owned();
        let running = SystemProbe.programs().expect("the programs");
        assert!(is_running(&running, &name), "{name} not in {running:?}");
    }

    #[test]
    fn a_page_is_read_at_most_once_a_minute() {
        let mut times = PageTimes::default();
        let id = RoutineId::generate();
        assert!(times.due(&id, 0, 60_000));
        assert!(!times.due(&id, 59_999, 60_000));
        assert!(times.due(&id, 60_000, 60_000));
        times.forget(&id);
        assert!(times.due(&id, 60_001, 60_000));
    }
}
