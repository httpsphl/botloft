//! What a bot left running in the background (spec 33.4): the live list
//! Claude Code sends whenever it changes, and stopping one of its tasks.
//! Nothing is stored: the tasks end with the process, and so does the list.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    BackgroundKind, BackgroundTask, BotBackgroundChanged, BotsStopTaskParams,
};
use serde_json::Value;
use tracing::debug;

use crate::chat::control;
use crate::service::bots::active;
use crate::service::{ApiError, ApiResult};
use crate::state::{Daemon, Event};

/// The most tasks kept for one bot.
const MAX_TASKS: usize = 50;
/// The longest description kept, in characters.
const MAX_DESCRIPTION: usize = 300;

#[derive(Default)]
pub struct Backgrounds {
    bots: Mutex<HashMap<BotId, Vec<BackgroundTask>>>,
}

impl Backgrounds {
    fn lock(&self) -> MutexGuard<'_, HashMap<BotId, Vec<BackgroundTask>>> {
        self.bots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// What `bot` has running now; empty when nothing is.
    pub fn get(&self, bot: &BotId) -> Vec<BackgroundTask> {
        self.lock().get(bot).cloned().unwrap_or_default()
    }
}

/// `system/background_tasks_changed`: the whole live list, which replaces
/// the one before. Tasks Claude Code runs for itself (`ambient`) stay out.
pub(crate) fn changed(daemon: &Daemon, bot: &BotId, event: &Value) {
    let now = daemon.clock.now_ms();
    let listed = event["tasks"].as_array().map_or(&[][..], Vec::as_slice);
    replace(daemon, bot, |before| {
        listed
            .iter()
            .filter(|task| task["ambient"].as_bool() != Some(true))
            .filter_map(|task| {
                let id = task["task_id"].as_str().filter(|id| !id.is_empty())?;
                let seen_at = before
                    .iter()
                    .find(|known| known.task_id == id)
                    .map_or(now, |known| known.seen_at);
                Some(BackgroundTask {
                    task_id: id.to_owned(),
                    kind: kind_of(task["task_type"].as_str()),
                    description: task["description"]
                        .as_str()
                        .unwrap_or_default()
                        .chars()
                        .take(MAX_DESCRIPTION)
                        .collect(),
                    seen_at,
                })
            })
            .take(MAX_TASKS)
            .collect()
    });
}

fn kind_of(task_type: Option<&str>) -> BackgroundKind {
    match task_type {
        Some("local_bash") => BackgroundKind::Command,
        Some("local_agent") => BackgroundKind::Agent,
        Some("remote_agent") => BackgroundKind::Remote,
        _ => BackgroundKind::Other,
    }
}

/// The process is gone, and what it ran in the background with it.
pub(crate) fn process_ended(daemon: &Daemon, bot: &BotId) {
    replace(daemon, bot, |_| Vec::new());
}

/// Puts the new list in place and tells the app when it differs.
fn replace(
    daemon: &Daemon,
    bot: &BotId,
    make: impl FnOnce(&[BackgroundTask]) -> Vec<BackgroundTask>,
) {
    let tasks = {
        let mut bots = daemon.background.lock();
        let before = bots.get(bot).map_or(&[][..], Vec::as_slice);
        let after = make(before);
        if after == before {
            return;
        }
        if after.is_empty() {
            bots.remove(bot);
        } else {
            bots.insert(bot.clone(), after.clone());
        }
        after
    };
    debug!(bot = %bot, tasks = tasks.len(), "background tasks");
    daemon.emit(Event::BotBackground(BotBackgroundChanged {
        bot_id: bot.clone(),
        tasks,
    }));
}

/// `bots.stopTask`: asks the bot's process to end one of its tasks. It
/// answers with a new list without it (spec 33.4).
pub fn stop(daemon: &Daemon, params: BotsStopTaskParams) -> ApiResult<()> {
    let store = daemon.store();
    let (_, record) = active(&store, &params.bot_id)?;
    let known = daemon
        .background
        .get(&record.id)
        .iter()
        .any(|task| task.task_id == params.task_id);
    if !known {
        return Err(ApiError::NotFound(format!(
            "bot {} has no task {} running",
            record.id, params.task_id
        )));
    }
    daemon
        .supervisor
        .write_control(&record.id, control::stop_task_request(&params.task_id))
        .map_err(|_| ApiError::Conflict(format!("bot {} is not running", record.id)))
}
