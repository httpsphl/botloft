//! Closing the browsers nobody needs (spec 21.2): idle ones, and those of
//! bots that are paused or archived.

use std::sync::Arc;
use std::time::Duration;

use botloft_core::ids::BotId;

use super::{Browsers, Want, lock};
use crate::service::{bots, crews};
use crate::state::Daemon;

const SWEEP_EVERY: Duration = Duration::from_secs(30);

pub async fn run(daemon: Arc<Daemon>) {
    let mut tick = tokio::time::interval(SWEEP_EVERY);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tick.tick().await;
        daemon.browsers.sweep(|bot| want(&daemon, bot));
    }
}

fn want(daemon: &Daemon, bot: &BotId) -> Want {
    let store = daemon.store();
    let Ok(record) = bots::find(&store, bot) else {
        return Want::Forget;
    };
    let Ok(crew) = crews::find(&store, &record.crew_id) else {
        return Want::Forget;
    };
    if record.archived_at.is_some() || crew.archived_at.is_some() {
        Want::Forget
    } else if record.paused || crew.paused {
        Want::Close
    } else {
        Want::Keep
    }
}

impl Browsers {
    /// Closes the browsers nobody used or watched for a while, and those
    /// of bots that should not run (spec 21.2). Browsers in a call stay.
    pub fn sweep(&self, want: impl Fn(&BotId) -> Want) {
        let open: Vec<(BotId, bool)> = lock(&self.slots)
            .iter()
            .filter(|(_, slot)| slot.session.is_some() && slot.calls.try_lock().is_ok())
            .map(|(bot, slot)| {
                let unused = slot.watchers == 0 && slot.used.elapsed() >= self.settings.idle;
                (bot.clone(), unused)
            })
            .collect();
        for (bot, unused) in open {
            match want(&bot) {
                Want::Forget => self.forget(&bot),
                Want::Close => self.close(&bot),
                Want::Keep if unused => self.close(&bot),
                Want::Keep => {}
            }
        }
    }

    /// Makes room for one more browser by closing the one idle longest;
    /// never one in the owner's hands.
    pub(super) fn make_room(&self, starting: &BotId) {
        let victim = {
            let slots = lock(&self.slots);
            let open = slots.values().filter(|slot| slot.session.is_some()).count();
            if open < self.settings.max_open {
                return;
            }
            slots
                .iter()
                .filter(|(bot, slot)| {
                    *bot != starting
                        && slot.session.is_some()
                        && slot.calls.try_lock().is_ok()
                        && slot.held.borrow().is_none()
                })
                .min_by_key(|(_, slot)| slot.used)
                .map(|(bot, _)| bot.clone())
        };
        if let Some(bot) = victim {
            self.close(&bot);
        }
    }
}
