//! `bots.delete` and `crews.delete` (spec 7.6): removing a bot or a crew
//! for good, active or archived. Archiving keeps everything in the
//! database; deleting keeps only the folders on disk (spec 5).

use botloft_core::ids::{BotId, MessageId};
use botloft_core::protocol::{
    BotDeleted, BotIdParams, Crew, CrewDeleted, CrewIdParams, Message, MessageKind, SenderKind,
};
use botloft_store::{BotRecord, Store, TaskFilter};
use tracing::{info, warn};

use super::messages::post;
use super::tasks::UNFINISHED;
use super::{ApiResult, bots, crews, lead};
use crate::state::{Daemon, Event};

/// Deletes the bot: its process stops at once and its conversation,
/// routines and tasks leave the database. Its folder stays.
pub fn bot(daemon: &Daemon, params: BotIdParams) -> ApiResult<BotDeleted> {
    let store = daemon.store();
    let record = bots::find(&store, &params.bot_id)?;
    let crew = crews::find(&store, &record.crew_id)?;
    let notices = task_notices(&store, &crew, &record)?;
    let waiting = store.pending_approvals(&record.id)?;
    store.delete_bot(&record.id)?;
    stop(daemon, &record.id);

    if lead::is_lead(&crew, &record.id) {
        // The crew lost its chief with the bot.
        crews::changed(daemon, crews::find(&store, &crew.id)?);
    }
    let deleted = BotDeleted {
        bot_id: record.id,
        crew_id: crew.id.clone(),
    };
    daemon.emit(Event::BotDeleted(deleted.clone()));
    for (to, body) in notices {
        tell(daemon, &store, &crew, to, body);
    }
    drop(store);
    daemon.approvals.forget(&waiting);
    release(daemon, &deleted.bot_id);
    info!(bot = %deleted.bot_id, "bot deleted");
    Ok(deleted)
}

/// Deletes the crew with every bot in it, archived ones too. Its folders
/// stay.
pub fn crew(daemon: &Daemon, params: CrewIdParams) -> ApiResult<CrewDeleted> {
    let store = daemon.store();
    let crew = crews::find(&store, &params.crew_id)?;
    let bots = store.bots(Some(&crew.id), true)?;
    let mut waiting = Vec::new();
    for bot in &bots {
        waiting.extend(store.pending_approvals(&bot.id)?);
    }
    store.delete_crew(&crew.id)?;
    for bot in &bots {
        stop(daemon, &bot.id);
    }
    let deleted = CrewDeleted { crew_id: crew.id };
    daemon.emit(Event::CrewDeleted(deleted.clone()));
    drop(store);
    daemon.approvals.forget(&waiting);
    for bot in &bots {
        release(daemon, &bot.id);
    }
    info!(crew = %deleted.crew_id, bots = bots.len(), "crew deleted");
    Ok(deleted)
}

/// Kills the bot's process and closes its browser, profile included.
fn stop(daemon: &Daemon, bot: &BotId) {
    daemon.supervisor.forget(bot);
    daemon.browsers.forget(bot);
}

/// Lets go of what the daemon kept in memory for the bot. Called without
/// the store: the routines take their own lock before it.
fn release(daemon: &Daemon, bot: &BotId) {
    daemon.routines.forget(bot);
    daemon.screens.forget(bot);
}

/// What the other bots should hear about the unfinished tasks a deleted
/// bot leaves (spec 7.6): who to tell, and what. Only bots that still run
/// are told.
fn task_notices(store: &Store, crew: &Crew, bot: &BotRecord) -> ApiResult<Vec<(BotId, String)>> {
    if crew.archived_at.is_some() {
        return Ok(Vec::new());
    }
    let handle = &bot.handle;
    let owed = store.tasks(TaskFilter {
        assignee: Some(&bot.id),
        statuses: &UNFINISHED,
        ..TaskFilter::default()
    })?;
    let asked = store.tasks(TaskFilter {
        requester: Some(&bot.id),
        statuses: &UNFINISHED,
        ..TaskFilter::default()
    })?;
    let owed = owed.into_iter().rev().map(|task| {
        let body = format!(
            "Task {} for @{handle} will not be done: the owner deleted @{handle}. Get it done \
             another way, or tell the owner what is missing.",
            task.id
        );
        (task.requester_bot_id, body)
    });
    let asked = asked.into_iter().rev().map(|task| {
        let body = format!(
            "Task {} from @{handle} no longer needs a result: the owner deleted @{handle}. \
             Stop working on it.",
            task.id
        );
        (task.assignee_bot_id, body)
    });
    let mut notices = Vec::new();
    for (to, body) in owed.chain(asked) {
        if store
            .bot(&to)?
            .is_some_and(|other| other.archived_at.is_none())
        {
            notices.push((to, body));
        }
    }
    Ok(notices)
}

/// A notice from Botloft to a bot of the crew.
fn tell(daemon: &Daemon, store: &Store, crew: &Crew, to: BotId, body: String) {
    let notice = Message {
        id: MessageId::generate(),
        crew_id: crew.id.clone(),
        from_kind: SenderKind::System,
        from_bot_id: None,
        to_bot_id: to,
        kind: MessageKind::System,
        body,
        task_id: None,
        routine_id: None,
        attachments: Vec::new(),
        created_at: daemon.clock.now_ms(),
    };
    if let Err(err) = post(daemon, store, notice, None) {
        warn!(crew = %crew.id, "could not tell a bot about a deleted bot's task: {err}");
    }
}
