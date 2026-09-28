//! `terminal.*` operations (spec 8). Attaching needs the connection, so the
//! RPC layer streams the output; this module validates and prepares it.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::ids::BotId;
use botloft_core::protocol::{TerminalAttachParams, TerminalResizeParams, TerminalWriteParams};
use bytes::Bytes;

use super::{ApiError, ApiResult, bots};
use crate::runtime::TermSize;
use crate::state::Daemon;
use crate::terminal::Attachment;

/// Largest terminal the daemon accepts.
const MAX_CELLS: u16 = 1000;

pub fn attach(daemon: &Daemon, params: TerminalAttachParams) -> ApiResult<Attachment> {
    ensure_exists(daemon, &params.bot_id)?;
    let terminal = daemon.supervisor.terminal(&params.bot_id);
    Ok(terminal.attach(params.generation, params.offset))
}

pub fn write(daemon: &Daemon, params: TerminalWriteParams) -> ApiResult<()> {
    ensure_exists(daemon, &params.bot_id)?;
    let data = BASE64
        .decode(params.data.as_bytes())
        .map_err(|_| ApiError::validation("data must be base64"))?;
    daemon
        .supervisor
        .write(&params.bot_id, Bytes::from(data))
        .map_err(|err| ApiError::Conflict(err.to_string()))
}

pub fn resize(daemon: &Daemon, params: TerminalResizeParams) -> ApiResult<()> {
    ensure_exists(daemon, &params.bot_id)?;
    let valid = |n: u16| (1..=MAX_CELLS).contains(&n);
    if !valid(params.cols) || !valid(params.rows) {
        return Err(ApiError::validation(format!(
            "cols and rows must be between 1 and {MAX_CELLS}"
        )));
    }
    let size = TermSize {
        cols: params.cols,
        rows: params.rows,
    };
    daemon.supervisor.resize(&params.bot_id, size);
    Ok(())
}

/// Base64 for `terminal.data`.
pub fn encode(data: &[u8]) -> String {
    BASE64.encode(data)
}

fn ensure_exists(daemon: &Daemon, bot: &BotId) -> ApiResult<()> {
    bots::find(&daemon.store(), bot).map(|_| ())
}
