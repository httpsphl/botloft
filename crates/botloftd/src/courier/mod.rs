//! The courier (spec 9.1): moves pending deliveries into the bots' inboxes,
//! in order and one at a time per bot, retrying with backoff until they are
//! sent or give up. Each cycle also expires overdue tasks (spec 9.4).

pub mod fake;
pub mod inbox;
mod settings;

use std::sync::Arc;
use std::time::Duration;

use botloft_core::envelope::{Envelope, Sender};
use botloft_core::protocol::{BotState, Delivery, SenderKind};
use botloft_store::{DeliveryOutcome, Store, StoreError};
use bytes::Bytes;
use tokio::sync::Notify;
use tracing::{debug, warn};

pub use self::inbox::{InboxWriter, PipeInbox};
pub use self::settings::CourierSettings;
use crate::clock;
use crate::service::tasks;
use crate::state::{Daemon, Event};
use crate::supervisor::Inbox;

/// How long to wait for a bot that cannot take messages yet (spec 9.1).
const WAIT_FOR_BOT: Duration = Duration::from_secs(5);
/// One attempt: opening the pipe, waiting while it is busy and writing.
const SEND_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Courier {
    settings: CourierSettings,
    writer: Arc<dyn InboxWriter>,
    wake: Notify,
}

impl Courier {
    pub fn new(settings: CourierSettings, writer: Arc<dyn InboxWriter>) -> Self {
        Self {
            settings,
            writer,
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
    Send(Job),
    /// The bot cannot take messages yet.
    Wait,
    /// It can never be delivered.
    Drop(&'static str),
}

struct Job {
    delivery: Delivery,
    inbox: Inbox,
    payload: Bytes,
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
        let step = match prepare(daemon, &store, &delivery, now) {
            Ok(step) => step,
            Err(err) => {
                warn!(delivery = %delivery.id, "courier could not prepare a delivery: {err}");
                continue;
            }
        };
        let finished = match step {
            Step::Send(job) => {
                let lease_until = clock::after(now, daemon.courier.settings.lease);
                match store.claim_delivery(&job.delivery.id, now, lease_until) {
                    Ok(Some(sending)) => {
                        daemon.emit(Event::DeliveryChanged(sending));
                        tokio::spawn(send(Arc::clone(daemon), job));
                    }
                    Ok(None) => {}
                    Err(err) => warn!(delivery = %delivery.id, "courier could not claim: {err}"),
                }
                continue;
            }
            Step::Wait => {
                let until = clock::after(now, WAIT_FOR_BOT);
                store.finish_delivery(&delivery.id, DeliveryOutcome::Defer { until }, now)
            }
            Step::Drop(error) => {
                store.finish_delivery(&delivery.id, DeliveryOutcome::Dead { error }, now)
            }
        };
        match finished {
            Ok(Some(changed)) if changed.state != delivery.state => {
                debug!(delivery = %changed.id, "delivery dropped");
                daemon.emit(Event::DeliveryChanged(changed));
            }
            Ok(_) => {}
            Err(err) => warn!(delivery = %delivery.id, "courier could not update: {err}"),
        }
    }
}

/// Decides whether the delivery can go now and renders its envelope.
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
    let Some(inbox) = daemon.supervisor.inbox(&bot.id).filter(|_| ready) else {
        return Ok(Step::Wait);
    };
    let task = match &message.task_id {
        Some(id) => store.task(id)?,
        None => None,
    };
    let sender = match (&message.from_kind, &message.from_bot_id) {
        (SenderKind::Bot, Some(id)) => store.bot(id)?.map(|sender| sender.handle),
        _ => None,
    };
    let from = match message.from_kind {
        SenderKind::Owner => Sender::Owner,
        SenderKind::Bot => Sender::Bot {
            handle: sender.as_deref().unwrap_or("unknown"),
        },
        SenderKind::System => Sender::Botloft,
    };
    let text = Envelope {
        from,
        crew: &crew.name,
        kind: message.kind,
        task: task.as_ref(),
        body: &message.body,
    }
    .render(now);
    Ok(Step::Send(Job {
        delivery: delivery.clone(),
        payload: inbox::payload(&inbox.token, &text),
        inbox,
    }))
}

/// One attempt, then the outcome in the database.
async fn send(daemon: Arc<Daemon>, job: Job) {
    let courier = &daemon.courier;
    let written = tokio::time::timeout(
        SEND_TIMEOUT,
        courier.writer.write(&job.inbox.socket, job.payload),
    )
    .await;
    let error = match written {
        Ok(Ok(())) => None,
        Ok(Err(err)) => Some(err.to_string()),
        Err(_) => Some("timed out writing to the inbox".to_owned()),
    };
    let now = daemon.clock.now_ms();
    let delivery = &job.delivery;
    let outcome = match &error {
        None => DeliveryOutcome::Sent,
        // The bot restarted meanwhile: its old inbox is gone, which says
        // nothing about this delivery. Try the new one as soon as it is up.
        Some(_) if daemon.supervisor.inbox(&delivery.bot_id).as_ref() != Some(&job.inbox) => {
            DeliveryOutcome::Defer { until: now }
        }
        Some(error) => {
            let attempts = delivery.attempts + 1;
            let retry_at = (attempts < courier.settings.max_attempts)
                .then(|| clock::after(now, courier.settings.retry_delay(attempts)));
            DeliveryOutcome::Failed { error, retry_at }
        }
    };
    if let Some(error) = &error {
        debug!(delivery = %delivery.id, bot = %delivery.bot_id, "delivery attempt failed: {error}");
    }
    let finished = daemon.store().finish_delivery(&delivery.id, outcome, now);
    match finished {
        Ok(Some(changed)) => {
            if let DeliveryOutcome::Failed { retry_at: None, .. } = outcome {
                warn!(delivery = %changed.id, bot = %changed.bot_id, "gave up on a delivery");
            }
            daemon.emit(Event::DeliveryChanged(changed));
        }
        Ok(None) => {}
        Err(err) => warn!(delivery = %delivery.id, "courier could not record a send: {err}"),
    }
    courier.wake();
}
