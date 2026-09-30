//! The courier (spec 9.1): writes pending deliveries into the bots' stdin,
//! in order and one at a time per bot, and puts back what a process that
//! ended never began. Each cycle also expires overdue tasks (spec 9.4).

mod render;
mod settings;

use std::sync::Arc;
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotState, Delivery, Message, SenderKind};
use botloft_store::{DeliveryOutcome, Store, StoreError};
use tokio::sync::Notify;
use tracing::{debug, warn};

use self::render::{Context, Rendered, render};
pub use self::settings::CourierSettings;
use crate::service::tasks;
use crate::state::{Daemon, Event};
use crate::{clock, routines};

/// How long to wait for a bot that cannot take messages yet (spec 9.1).
const WAIT_FOR_BOT: Duration = Duration::from_secs(5);
/// Why a delivery goes back in the queue after its process ended.
const PROCESS_ENDED: &str = "the bot's process ended before it began this message";

pub struct Courier {
    settings: CourierSettings,
    wake: Notify,
}

impl Courier {
    pub fn new(settings: CourierSettings) -> Self {
        Self {
            settings,
            wake: Notify::new(),
        }
    }

    /// Runs a cycle now instead of at the next poll.
    pub fn wake(&self) {
        self.wake.notify_one();
    }
}

/// Delivers until the daemon stops. Leases left by a previous run are
/// recovered on the first cycle.
pub async fn run(daemon: Arc<Daemon>) {
    loop {
        cycle(&daemon);
        tokio::select! {
            () = tokio::time::sleep(daemon.courier.settings.poll_interval) => {}
            () = daemon.courier.wake.notified() => {}
        }
    }
}

/// What to do with a due delivery.
enum Step {
    Send(Rendered),
    /// The bot cannot take messages yet.
    Wait,
    /// It can never be delivered.
    Drop(&'static str),
}

fn cycle(daemon: &Arc<Daemon>) {
    let now = daemon.clock.now_ms();
    let store = daemon.store();
    match store.recover_leases(now) {
        Ok(recovered) => {
            for delivery in recovered {
                debug!(delivery = %delivery.id, "a send was cut off; delivering again");
                daemon.emit(Event::DeliveryChanged(delivery));
            }
        }
        Err(err) => warn!("courier could not recover leases: {err}"),
    }
    tasks::expire_overdue(daemon, &store, now);
    let due = match store.due_deliveries(now) {
        Ok(due) => due,
        Err(err) => {
            warn!("courier could not read the queue: {err}");
            return;
        }
    };
    for delivery in due {
        let changed = match prepare(daemon, &store, &delivery, now) {
            Ok(Step::Send(rendered)) => send(daemon, &store, &delivery, &rendered, now),
            Ok(Step::Wait) => {
                let until = clock::after(now, WAIT_FOR_BOT);
                finish(&store, &delivery, DeliveryOutcome::Defer { until }, now)
            }
            Ok(Step::Drop(error)) => {
                finish(&store, &delivery, DeliveryOutcome::Dead { error }, now)
            }
            Err(err) => {
                warn!(delivery = %delivery.id, "courier could not prepare a delivery: {err}");
                continue;
            }
        };
        if let Some(changed) = changed.filter(|changed| changed.state != delivery.state) {
            daemon.emit(Event::DeliveryChanged(changed));
        }
    }
}

fn finish(
    store: &Store,
    delivery: &Delivery,
    outcome: DeliveryOutcome<'_>,
    now: i64,
) -> Option<Delivery> {
    store
        .finish_delivery(&delivery.id, outcome, now)
        .unwrap_or_else(|err| {
            warn!(delivery = %delivery.id, "courier could not update: {err}");
            None
        })
}

/// Claims the delivery, writes it and records how that went. `None` when
/// someone else claimed it first.
fn send(
    daemon: &Daemon,
    store: &Store,
    delivery: &Delivery,
    rendered: &Rendered,
    now: i64,
) -> Option<Delivery> {
    let lease_until = clock::after(now, daemon.courier.settings.lease);
    match store.claim_delivery(&delivery.id, now, lease_until) {
        Ok(Some(_)) => {}
        Ok(None) => return None,
        Err(err) => {
            warn!(delivery = %delivery.id, "courier could not claim: {err}");
            return None;
        }
    }
    let outcome = match daemon.supervisor.write_message(
        &delivery.bot_id,
        &rendered.uuid,
        rendered.line.clone(),
    ) {
        Ok(generation) => {
            debug!(delivery = %delivery.id, bot = %delivery.bot_id, generation, "delivery written");
            DeliveryOutcome::Sent {
                generation,
                turn_uuid: &rendered.uuid,
            }
        }
        // The process went away between the check and the write: not this
        // delivery's fault.
        Err(_) => DeliveryOutcome::Defer {
            until: clock::after(now, WAIT_FOR_BOT),
        },
    };
    finish(store, delivery, outcome, now)
}

/// Decides whether the delivery can go now and renders it.
fn prepare(
    daemon: &Daemon,
    store: &Store,
    delivery: &Delivery,
    now: i64,
) -> Result<Step, StoreError> {
    let Some(message) = store.message(&delivery.message_id)? else {
        return Ok(Step::Drop("the message no longer exists"));
    };
    let Some(bot) = store.bot(&delivery.bot_id)? else {
        return Ok(Step::Drop("the bot no longer exists"));
    };
    let Some(crew) = store.crew(&bot.crew_id)? else {
        return Ok(Step::Drop("the crew no longer exists"));
    };
    if bot.archived_at.is_some() || crew.archived_at.is_some() {
        return Ok(Step::Drop("the bot was archived"));
    }
    let ready = matches!(
        daemon.supervisor.status(&bot.id),
        Some((BotState::Idle | BotState::Busy | BotState::NeedsApproval, _))
    );
    if !ready {
        return Ok(Step::Wait);
    }
    let task = match &message.task_id {
        Some(id) => store.task(id)?,
        None => None,
    };
    let sender = match (&message.from_kind, &message.from_bot_id) {
        (SenderKind::Bot, Some(id)) => store.bot(id)?.map(|sender| sender.handle),
        _ => None,
    };
    let workspace = daemon.paths.bot_workspace(&crew.slug, &bot.slug);
    let routine = routine_context(store, &message)?;
    let context = Context {
        crew_name: &crew.name,
        sender_handle: sender.as_deref(),
        task: task.as_ref(),
        workspace: &workspace,
        routine: routine
            .as_ref()
            .map(|(name, scheduled, zone)| (name.as_str(), scheduled.as_str(), zone.as_str())),
    };
    Ok(Step::Send(render(&message, &context, now)))
}

/// A routine's name, the local time the run is for and the zone.
fn routine_context(
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

/// The process of `generation` ended: what it never began goes back in
/// the queue, counting an attempt (spec 9.1).
pub fn requeue_unread(daemon: &Daemon, bot: &BotId, generation: u64) {
    let now = daemon.clock.now_ms();
    let store = daemon.store();
    let unread = match store.unread_deliveries(bot, generation) {
        Ok(unread) => unread,
        Err(err) => {
            warn!(bot = %bot, "could not read unread deliveries: {err}");
            return;
        }
    };
    let settings = &daemon.courier.settings;
    for delivery in unread {
        let attempts = delivery.attempts + 1;
        let retry_at = (attempts < settings.max_attempts)
            .then(|| clock::after(now, settings.retry_delay(attempts)));
        match store.reopen_unread(&delivery.id, PROCESS_ENDED, retry_at, now) {
            Ok(Some(changed)) => {
                if retry_at.is_none() {
                    warn!(delivery = %changed.id, bot = %bot, "gave up on a delivery");
                }
                daemon.emit(Event::DeliveryChanged(changed));
            }
            Ok(None) => {}
            Err(err) => warn!(delivery = %delivery.id, "could not requeue: {err}"),
        }
    }
    drop(store);
    daemon.courier.wake();
}
