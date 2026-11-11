//! What the courier reads besides the message to write its line: the
//! routine that sent it (spec 20.5) or the question it answers (spec 23.4).

use botloft_core::envelope::{RoutineEnvelope, RoutineWhen};
use botloft_core::protocol::{ChatBody, Message, Question, Schedule, WaitEnd, WaitOn};
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
    /// What a wait waited for, how it ended and when it ran out, local.
    Wait {
        on: WaitOn,
        ended: WaitEnd,
        until: String,
        timezone: String,
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
            Trigger::Wait {
                on,
                ended,
                until,
                timezone,
            } => RoutineWhen::Wait {
                on,
                ended: *ended,
                until,
                timezone,
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
    let plan = routines::schedule::Plan::new(&routine.schedule, &routine.timezone);
    let trigger = match (run.signal, run.waited, &routine.schedule) {
        (_, Some(ended), Schedule::Wait { on, until }) => Trigger::Wait {
            on: on.clone(),
            ended,
            until: plan
                .as_ref()
                .map(|plan| plan.local_time(*until))
                .unwrap_or_default(),
            timezone: routine.timezone.clone(),
        },
        (Some(signal), _, _) => Trigger::Signal {
            from: match &signal.from_bot_id {
                Some(bot) => store.bot(bot)?.map(|sender| sender.handle),
                None => None,
            },
            name: signal.name,
            note: signal.note,
        },
        _ => Trigger::Scheduled {
            at: plan
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

/// The id and title of the card the owner's message answers (spec 8.11).
pub(super) fn card(
    store: &Store,
    message: &Message,
) -> Result<Option<(String, String)>, StoreError> {
    let Some(reply) = &message.reply_to else {
        return Ok(None);
    };
    Ok(store
        .chat_item(&reply.item_id)?
        .and_then(|item| match item.body {
            ChatBody::Card(card) => Some((item.id.to_string(), card.title)),
            _ => None,
        }))
}

/// The question a message answers, or the note it replies to.
pub(super) fn question(store: &Store, message: &Message) -> Result<Option<Question>, StoreError> {
    let Some(id) = &message.question_id else {
        return Ok(None);
    };
    Ok(store.question(id)?.map(|record| record.question))
}
