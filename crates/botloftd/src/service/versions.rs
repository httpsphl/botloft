//! `versions.*` (spec 8.10): the history of a bot's memory and instructions,
//! and going back to one of its versions. Going back saves a new version, so
//! it can be undone in turn.

use std::fs;

use botloft_core::protocol::{
    BotsUpdateParams, TextVersion, VersionAuthor, VersionKind, VersionsListParams,
    VersionsRestoreParams,
};

use super::{ApiError, ApiResult, bots};
use crate::state::Daemon;
use crate::versions::{self, Origin};

/// The most versions one call returns.
const LIST_MAX: usize = 100;

pub fn list(daemon: &Daemon, params: VersionsListParams) -> ApiResult<Vec<TextVersion>> {
    bots::active(&daemon.store(), &params.bot_id)?;
    if params.kind == VersionKind::Memory {
        // The newest version is the memory as it is now.
        versions::look_between_turns(daemon, &params.bot_id);
    }
    Ok(daemon
        .store()
        .versions(&params.bot_id, params.kind, LIST_MAX)?)
}

pub fn restore(daemon: &Daemon, params: VersionsRestoreParams) -> ApiResult<TextVersion> {
    let version = daemon
        .store()
        .version(&params.version_id)?
        .ok_or_else(|| ApiError::NotFound(format!("version {} not found", params.version_id)))?;
    bots::active(&daemon.store(), &version.bot_id)?;
    let origin = Origin {
        author: VersionAuthor::Owner,
        restored_from: Some(version.id.clone()),
    };
    match version.kind {
        VersionKind::Memory => {
            let path = versions::memory_path(daemon, &version.bot_id)
                .ok_or_else(|| ApiError::NotFound(format!("bot {} not found", version.bot_id)))?;
            // What is there now is kept first, so going back can be undone.
            versions::look_between_turns(daemon, &version.bot_id);
            fs::write(&path, &version.text).map_err(ApiError::Workspace)?;
            versions::record(
                daemon,
                &version.bot_id,
                VersionKind::Memory,
                version.text.clone(),
                origin,
            );
        }
        VersionKind::Instructions => {
            bots::update_by(
                daemon,
                BotsUpdateParams {
                    bot_id: version.bot_id.clone(),
                    name: None,
                    role: None,
                    instructions: Some(version.text.clone()),
                    color: None,
                    allowed_commands: None,
                    plan_first: None,
                },
                origin,
            )?;
        }
    }
    daemon
        .store()
        .latest_version(&version.bot_id, version.kind)?
        .ok_or_else(|| ApiError::NotFound(format!("bot {} has no versions", version.bot_id)))
}
