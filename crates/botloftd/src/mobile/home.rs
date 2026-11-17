//! The phone's Home (spec 28.15): the owner's shortcuts, which it runs with
//! a tap, and what the crews did since the phone's midnight.

use std::sync::Arc;

use botloft_core::protocol::{
    ActivityTodayParams, FromPhone, PHONE_SHORTCUTS_MAX, PhoneShortcut, RoutineIdParams, Schedule,
    ToPhone,
};

use super::Mobile;
use super::bridge::blocking_result;
use crate::service::{ApiError, bots, island, routines};
use crate::state::Daemon;

/// The routines that wait for the owner's tap (spec 20.2), of agents that
/// are active, by name.
fn shortcuts(daemon: &Daemon) -> Vec<PhoneShortcut> {
    let store = daemon.store();
    let mut found: Vec<PhoneShortcut> = store
        .routines(None, false)
        .unwrap_or_default()
        .into_iter()
        .filter(|routine| {
            matches!(routine.schedule, Schedule::Manual)
                && routine.enabled
                && routine.archived_at.is_none()
                && bots::active(&store, &routine.bot_id).is_ok()
        })
        .map(|routine| PhoneShortcut {
            routine_id: routine.id,
            name: routine.name,
            bot_id: routine.bot_id,
        })
        .collect();
    found.sort_by_key(|shortcut| shortcut.name.to_lowercase());
    found.truncate(PHONE_SHORTCUTS_MAX);
    found
}

/// Runs the routine if it is one of the owner's shortcuts; the phone runs
/// nothing else.
fn run(daemon: &Daemon, params: RoutineIdParams) -> Result<(), &'static str> {
    let shortcut = shortcuts(daemon)
        .iter()
        .any(|shortcut| shortcut.routine_id == params.routine_id);
    if !shortcut {
        return Err("not_found");
    }
    routines::run_now(daemon, params)
        .map(|_| ())
        .map_err(|error| match error {
            ApiError::NotFound(_) | ApiError::Conflict(_) => "not_found",
            _ => "failed",
        })
}

impl Mobile {
    pub(crate) async fn home_apply(&self, daemon: &Arc<Daemon>, phone: &str, message: FromPhone) {
        match message {
            FromPhone::Home { since } => {
                let home = blocking_result(daemon, move |daemon| ToPhone::Home {
                    shortcuts: shortcuts(daemon),
                    today: island::today(daemon, ActivityTodayParams { since }).ok(),
                })
                .await;
                if let Some(home) = home {
                    self.send_to(daemon, phone, &home);
                }
            }
            FromPhone::RoutineRun {
                client_id,
                routine_id,
            } => {
                let result = if self.may_send(daemon, phone) {
                    blocking_result(daemon, move |daemon| {
                        run(daemon, RoutineIdParams { routine_id })
                    })
                    .await
                    .unwrap_or(Err("failed"))
                } else {
                    Err("rate_limited")
                };
                self.send_to(
                    daemon,
                    phone,
                    &ToPhone::Ran {
                        client_id,
                        ok: result.is_ok(),
                        reason: result.err().map(str::to_owned),
                    },
                );
            }
            _ => {}
        }
    }
}
