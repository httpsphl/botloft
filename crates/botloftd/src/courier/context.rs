//! What the courier reads besides the message to write its line: the
//! routine that sent it (spec 20.5) or the question it answers (spec 23.4).

use botloft_core::protocol::Message;
use botloft_store::{Store, StoreError};

use crate::routines;

/// A routine's name, the local time the run is for and the zone.
pub(super) fn routine(
    store: &Store,
    message: &Message,
) -> Result<Option<(String, String, String)>, StoreError> {
    let Some(id) = &message.routine_id else {
        return Ok(None);
    };
    let (Some(routine), Some(run)) = (store.routine(id)?, store.run_of_message(&message.id)?)
    else {
        return Ok(None);
    };
    let scheduled = routines::schedule::Plan::new(&routine.schedule, &routine.timezone)
        .map(|plan| plan.local_time(run.scheduled_for))
        .unwrap_or_default();
    Ok(Some((routine.name, scheduled, routine.timezone)))
}

/// The id and text of the question a message answers.
pub(super) fn question(
    store: &Store,
    message: &Message,
) -> Result<Option<(String, String)>, StoreError> {
    let Some(id) = &message.question_id else {
        return Ok(None);
    };
    Ok(store
        .question(id)?
        .map(|record| (record.question.id.to_string(), record.question.text)))
}
