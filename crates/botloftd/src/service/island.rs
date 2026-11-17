//! `cards.list` and `activity.today` (spec 31): what the island shows.

use botloft_core::protocol::{ActivityTodayParams, Cards, DayActivity};

use super::{ApiError, ApiResult, key_requests};
use crate::mobile::cards;
use crate::state::Daemon;

/// The requests and questions waiting, as the phone's cards, with no
/// account or phone needed; and the requests for a key, which only the app
/// lists (spec 32.5).
pub fn cards(daemon: &Daemon) -> Cards {
    let (approvals, questions) = cards::waiting(daemon);
    Cards {
        approvals,
        questions,
        keys: key_requests::waiting(daemon),
    }
}

/// What every crew did since the app's local midnight.
pub fn today(daemon: &Daemon, params: ActivityTodayParams) -> ApiResult<DayActivity> {
    if params.since < 0 {
        return Err(ApiError::validation("since must not be negative"));
    }
    Ok(daemon.store().day_activity(params.since)?)
}
