//! What the courier reads besides the message to write its line: the
//! routine that sent it (spec 20.5) or the question it answers (spec 23.4).

use botloft_core::envelope::{RoutineEnvelope, RoutineWhen};
use botloft_core::protocol::Message;
use botloft_store::{Store, StoreError};

use crate::routines;

/// The routine that sent a message and what made it run.
pub(super) struct RoutineContext {
    name: String,
    trigger: Trigger,
}

enum Trigger {
    /// The local time the run is for, and the zone.
    Scheduled { at: String, timezone: String },
    /// The signal, the sender's handle (gone with a deleted bot) and the note.
    Signal {
        name: String,
        from: Option<String>,
        note: Option<String>,
    },
}

impl RoutineContext {
    pub(super) fn envelope<'a>(&'a self, body: &'a str) -> RoutineEnvelope<'a> {
        let when = match &self.trigger {
            Trigger::Scheduled { at, timezone } => RoutineWhen::Scheduled { at, timezone },
            Trigger::Signal { name, from, note } => RoutineWhen::Signal {
                name,
                from: from.as_deref(),
                note: note.as_deref(),
            },
        };
        RoutineEnvelope {
            name: &self.name,
            when,
            body,
        }
    }
}

/// The routine that sent `message`, if one did.
pub(super) fn routine(
    store: &Store,
    message: &Message,
) -> Result<Option<RoutineContext>, StoreError> {
    let Some(id) = &message.routine_id else {
        return Ok(None);
    };
    let (Some(routine), Some(run)) = (store.routine(id)?, store.run_of_message(&message.id)?)
    else {
        return Ok(None);
    };
    let trigger = match run.signal {
        Some(signal) => Trigger::Signal {
            from: match &signal.from_bot_id {
                Some(bot) => store.bot(bot)?.map(|sender| sender.handle),
                None => None,
            },
            name: signal.name,
            note: signal.note,
        },
        None => Trigger::Scheduled {
            at: routines::schedule::Plan::new(&routine.schedule, &routine.timezone)
                .map(|plan| plan.local_time(run.scheduled_for))
                .unwrap_or_default(),
            timezone: routine.timezone,
        },
    };
    Ok(Some(RoutineContext {
        name: routine.name,
        trigger,
    }))
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
