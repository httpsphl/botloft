//! `files.media` (spec 8.4): the address the app plays a sound or video of
//! the bot's from, of any size, served by `crate::media` in pieces.

use std::path::{Path, PathBuf};

use botloft_core::ids::BotId;
use botloft_core::protocol::{FilesReadParams, MediaLink};

use super::{file_name, media_type, places, reach, reached};
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

/// Whether a file of this type plays in the app's players.
fn plays(media: &str) -> bool {
    media.starts_with("audio/") || media.starts_with("video/")
}

/// `files.media`: a link to one of the files `files.read` would read, if it
/// is a sound or a video.
pub fn media(daemon: &Daemon, params: FilesReadParams) -> ApiResult<MediaLink> {
    let (_, places) = places(daemon, &params.bot_id)?;
    let (path, _) = reached(&places, &params.path)?;
    if !plays(media_type(&file_name(&path))) {
        return Err(ApiError::validation(
            "only sound and video files play from a link",
        ));
    }
    let token = daemon
        .media
        .link(&params.bot_id, &path)
        .ok_or_else(|| ApiError::Conflict("could not make a link to play the file".into()))?;
    Ok(MediaLink {
        url: format!("http://127.0.0.1:{}/media/{token}", daemon.port),
    })
}

/// Where one of the bot's files really is and its size, as `files.read`
/// would reach it now: `None` once it is gone or no longer the bot's.
pub(crate) fn located(daemon: &Daemon, bot: &BotId, path: &Path) -> Option<(PathBuf, u64)> {
    let (_, places) = places(daemon, bot).ok()?;
    let (path, meta) = reach(&places, path).ok()?;
    Some((path, meta.len()))
}
