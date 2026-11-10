//! `instructions.get` and `instructions.set` (spec 7.10): what the owner
//! writes in Settings for every agent to follow.

use botloft_core::protocol::OwnerInstructions;
use botloft_core::validate;
use tracing::info;

use super::ApiResult;
use crate::state::Daemon;

pub fn get(daemon: &Daemon) -> ApiResult<OwnerInstructions> {
    Ok(OwnerInstructions {
        text: daemon.store().owner_instructions()?,
    })
}

/// Saves them, and every agent reads them when it starts: one that is
/// running starts again, resuming its conversation, as soon as nothing is
/// in progress (spec 7.4). An agent asleep reads them when it wakes.
pub fn set(daemon: &Daemon, params: OwnerInstructions) -> ApiResult<OwnerInstructions> {
    let text = validate::instructions(params.text.trim())?;
    let before = daemon.store().owner_instructions()?;
    if text != before {
        daemon.store().set_owner_instructions(&text)?;
        // How long, never what they say.
        info!(
            chars = text.chars().count(),
            "owner's instructions for every agent changed"
        );
        daemon.supervisor.workspace_settings_changed();
    }
    Ok(OwnerInstructions { text })
}
