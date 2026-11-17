//! `builds.*` (spec 36.4): what the Maker built, and undoing it. Undoing
//! puts back what it changed in crews that already existed and archives
//! the crews the build made, with their bots and routines, so the owner can
//! still bring them back from the archive.

use botloft_core::protocol::{Build, BuildIdParams, CrewIdParams};
use tracing::info;

use super::{ApiError, ApiResult, crews, maker};
use crate::state::Daemon;

const LIST_LIMIT: u32 = 50;

/// The Maker's builds, newest first.
pub fn list(daemon: &Daemon) -> ApiResult<Vec<Build>> {
    let store = daemon.store();
    let Some(crew) = store.crews(true)?.into_iter().find(maker::is_maker_crew) else {
        return Ok(Vec::new());
    };
    let Some(bot) = store.bots(Some(&crew.id), true)?.into_iter().next() else {
        return Ok(Vec::new());
    };
    Ok(store.builds(&bot.id, LIST_LIMIT)?)
}

pub fn undo(daemon: &Daemon, params: BuildIdParams) -> ApiResult<Build> {
    let build = find(daemon, &params)?;
    if build.undone_at.is_some() {
        return Err(ApiError::Conflict(format!(
            "build {} was already undone",
            build.id
        )));
    }
    let now = daemon.clock.now_ms();
    if now > build.undo_until {
        return Err(ApiError::Conflict(format!(
            "build {} is more than a week old; archive its crews instead",
            build.id
        )));
    }
    let changes = daemon.store().build_changes(&build.id)?;
    crate::tools::undo_build_changes(daemon, &changes);
    for crew in &build.crew_ids {
        let exists = daemon.store().crew(crew)?.is_some();
        // A crew the owner deleted since is already gone.
        if exists {
            crews::archive(
                daemon,
                CrewIdParams {
                    crew_id: crew.clone(),
                },
            )?;
        }
    }
    if !daemon.store().undo_build(&build.id, now)? {
        return Err(ApiError::Conflict(format!(
            "build {} was already undone",
            build.id
        )));
    }
    info!(build = %build.id, "a build of the Maker was undone");
    find(daemon, &params)
}

fn find(daemon: &Daemon, params: &BuildIdParams) -> ApiResult<Build> {
    daemon
        .store()
        .build(&params.build_id)?
        .ok_or_else(|| ApiError::NotFound(format!("build {} does not exist", params.build_id)))
}
