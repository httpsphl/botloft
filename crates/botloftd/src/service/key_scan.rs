//! `keys.scan` and `keys.scrub` (spec 32.7): keys pasted into the
//! conversations before the owner kept them apart. A hit goes out with only
//! its start and end; scrubbing finds the value again by the hit's id.

use botloft_core::protocol::{KeyHit, KeysScrubParams, KeysSetParams};
use botloft_store::Pasted;
use tracing::info;

use super::{ApiError, ApiResult, keys};
use crate::state::Daemon;

/// What a deleted key reads as from then on.
pub const REMOVED: &str = "[chave removida]";

fn hit(pasted: Pasted) -> KeyHit {
    KeyHit {
        id: pasted.id,
        preview: pasted.preview,
        bot_ids: pasted.bots,
        places: pasted.places,
        last_at: pasted.last_at,
    }
}

pub fn scan(daemon: &Daemon) -> ApiResult<Vec<KeyHit>> {
    Ok(daemon.store().pasted_keys()?.into_iter().map(hit).collect())
}

/// Keeps or deletes each hit, then scans again.
pub fn scrub(daemon: &Daemon, params: KeysScrubParams) -> ApiResult<Vec<KeyHit>> {
    let found = daemon.store().pasted_keys()?;
    for scrub in params.hits {
        let pasted = found
            .iter()
            .find(|pasted| pasted.id == scrub.id)
            .ok_or_else(|| ApiError::NotFound(format!("hit {} is gone", scrub.id)))?;
        let mark = match scrub.keep {
            Some(keep) => {
                // Saved first: a name the daemon refuses leaves the text as it was.
                keys::set(
                    daemon,
                    KeysSetParams {
                        bot_id: keep.bot_id,
                        name: keep.name.clone(),
                        value: pasted.value.clone(),
                        description: None,
                        request_id: None,
                    },
                )?;
                format!("[chave {}]", keep.name.trim())
            }
            None => REMOVED.to_owned(),
        };
        let changed = daemon.store().replace_pasted(&pasted.value, &mark)?;
        info!(hit = %scrub.id, texts = changed, "pasted key scrubbed");
    }
    scan(daemon)
}
