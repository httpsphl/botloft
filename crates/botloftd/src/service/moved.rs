//! What came from another computer in a backup (spec 42.10): a crew's work
//! folder chosen there may be no path here, such as a Windows folder on a
//! Linux server. Such a crew goes back to its own folder, at the start,
//! before any agent runs. A folder that is only missing, such as a drive
//! not plugged in, stays chosen.

use std::path::Path;

use tracing::{info, warn};

use super::ApiResult;
use crate::state::Daemon;
use crate::workspace;

/// The crews moved to their own folder.
pub fn forget_foreign_folders(daemon: &Daemon) -> ApiResult<usize> {
    let store = daemon.store();
    let mut moved = 0;
    for mut crew in store.crews(true)? {
        if !crew.work_folder_chosen || Path::new(&crew.work_folder).is_absolute() {
            continue;
        }
        crew.work_folder_chosen = false;
        crew.work_folder = String::new();
        workspace::prepare_crew(&daemon.paths, &crew).map_err(super::ApiError::Workspace)?;
        store.update_crew(&crew)?;
        for bot in store.bots(Some(&crew.id), true)? {
            if let Err(err) = super::mcp::write_rules(daemon, &store, &crew, &bot) {
                warn!(bot = %bot.id, "could not refresh the bot rules after a folder change: {err}");
            }
        }
        info!(crew = %crew.id, "a folder from another computer: the crew works in its own folder");
        moved += 1;
    }
    Ok(moved)
}
