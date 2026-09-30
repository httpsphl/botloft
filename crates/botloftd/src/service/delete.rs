//! `bots.delete` and `crews.delete` (spec 7.6): removing a bot or a crew
//! for good, active or archived. Archiving keeps everything in the
//! database; deleting keeps only the folders on disk (spec 5), or sends
//! them to the Recycle Bin when the owner asks.

use botloft_core::ids::{BotId, MessageId};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use botloft_core::protocol::{
    BotDeleted, BotsDeleteParams, Crew, CrewDeleted, CrewsDeleteParams, FolderRecycled, Message,
    MessageKind, SenderKind,
};
use botloft_store::{BotRecord, Store, TaskFilter};
use tracing::{info, warn};

use super::messages::post;
use super::tasks::UNFINISHED;
use super::{ApiResult, bots, crews, lead};
use crate::state::{Daemon, Event};
use crate::{trash, workspace};

/// Deletes the bot: its process stops at once and its conversation,
/// routines and tasks leave the database. Its folder stays, unless the
/// owner asked for it to go to the Recycle Bin.
pub fn bot(daemon: &Daemon, params: BotsDeleteParams) -> ApiResult<BotDeleted> {
    let store = daemon.store();
    let record = bots::find(&store, &params.bot_id)?;
    let crew = crews::find(&store, &record.crew_id)?;
    let notices = task_notices(&store, &crew, &record)?;
    let waiting = store.pending_approvals(&record.id)?;
    let folder = daemon.paths.bot_workspace(&crew.slug, &record.slug);
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
    if params.recycle_folder == Some(true) {
        recycle(daemon, &crew, folder);
    }
    Ok(deleted)
}

/// Deletes the crew with every bot in it, archived ones too. Its folders
/// stay, unless the owner asked for the crew's own folder to go to the
/// Recycle Bin.
pub fn crew(daemon: &Daemon, params: CrewsDeleteParams) -> ApiResult<CrewDeleted> {
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
    let deleted = CrewDeleted {
        crew_id: crew.id.clone(),
    };
    daemon.emit(Event::CrewDeleted(deleted.clone()));
    drop(store);
    daemon.approvals.forget(&waiting);
    for bot in &bots {
        release(daemon, &bot.id);
    }
    info!(crew = %deleted.crew_id, bots = bots.len(), "crew deleted");
    if params.recycle_folder == Some(true) {
        recycle(daemon, &crew, daemon.paths.crew_dir(&crew.slug));
    }
    Ok(deleted)
}

/// Sends `folder`, which Botloft made for a deleted bot or crew, to the
/// Recycle Bin, and tells the apps how it went (spec 7.6). A work folder
/// the owner chose is theirs: a folder that holds it stays.
fn recycle(daemon: &Daemon, crew: &Crew, folder: PathBuf) {
    let chosen = crew
        .work_folder_chosen
        .then(|| Path::new(&crew.work_folder));
    // Only ever a folder under the workspaces, never the root of them.
    let refused = if !workspace::folder::contains(&daemon.paths.workspaces_root, &folder)
        || workspace::folder::contains(&folder, &daemon.paths.workspaces_root)
    {
        Some("it is not one of Botloft's folders")
    } else if chosen.is_some_and(|chosen| workspace::folder::contains(&folder, chosen)) {
        Some("the work folder you chose for the crew is inside it")
    } else {
        None
    };
    if let Some(reason) = refused {
        daemon.emit(Event::FolderRecycled(FolderRecycled {
            path: folder.to_string_lossy().into_owned(),
            error: Some(reason.to_owned()),
        }));
        return;
    }
    trash::move_away(Arc::clone(&daemon.trash), daemon.events(), folder);
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
