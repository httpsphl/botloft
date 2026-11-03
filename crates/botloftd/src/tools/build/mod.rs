//! `build` (spec 36.4): the Maker builds crews, their bots and routines in
//! one go. The owner checks the plan in the Maker's chat, may change or
//! leave out any part, and builds it; the call waits for that, like a
//! permission request. With "build without asking" on, it builds at once.
//! Either way the owner can undo it for a week.

mod apply;
pub(crate) mod plan;
mod schema;

use botloft_core::chat::BUILD_TOOL;
use botloft_core::ids::{ApprovalId, BotId};
use serde_json::{Value, json};

use self::plan::Plan;
use super::calls::{explain, parse, tool_result};
use crate::approvals::{self, Answer};
use crate::service::{ApiError, ApiResult, maker};
use crate::state::Daemon;

pub const BUILD: &str = "build";
pub(super) use self::schema::tool;

pub(super) async fn build(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    arguments: Value,
) -> Value {
    tool_result(run(daemon, bot, generation, arguments).await)
}

async fn run(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    arguments: Value,
) -> Result<Value, String> {
    if !maker::is_maker(&daemon.store(), bot).map_err(explain)? {
        return Err("only the Maker builds crews; the owner asks the Maker for one".to_owned());
    }
    let mut plan: Plan = parse(arguments)?;
    plan::resolve(daemon, &mut plan)?;
    plan::check(daemon, &plan)?;
    if waiting_build(daemon, bot)? {
        return Err(
            "another plan of yours still waits for the owner; wait for their answer".to_owned(),
        );
    }
    if daemon.settings.get().maker_builds_without_asking {
        let (_, report) = apply::build(daemon, bot, None, &plan)?;
        return Ok(built(
            report,
            "The owner lets you build without asking, so it was built at once: tell them what \
             you built, and that they can undo it from your chat for a week.",
        ));
    }

    let input = serde_json::to_value(&plan).map_err(|err| err.to_string())?;
    let mut opened: Option<ApprovalId> = None;
    let answer = approvals::ask_with(daemon, bot, generation, BUILD_TOOL, &input, "", |id| {
        opened = Some(id.clone());
    })
    .await;
    match answer {
        Some(Answer::Allowed { input, .. }) => {
            let (plan, how) = match input {
                None => (plan, "The owner built it as you planned."),
                Some(changed) => (
                    serde_json::from_str::<Plan>(&changed)
                        .map_err(|err| format!("the owner's changes could not be read: {err}"))?,
                    "The owner changed the plan before building it: only what is listed here \
                     was built.",
                ),
            };
            let (_, report) = apply::build(daemon, bot, opened, &plan)?;
            Ok(built(report, how))
        }
        Some(Answer::Denied { note }) => {
            let why = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Ok(json!({
                "built": false,
                "note": format!("The owner did not build this plan.{why} Nothing was made."),
            }))
        }
        Some(Answer::Expired) => Ok(json!({
            "built": false,
            "note": "The owner did not answer in time. Nothing was made; offer it again later.",
        })),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}

fn built(report: Value, how: &str) -> Value {
    json!({
        "built": true,
        "crews": report,
        "note": format!(
            "{how} The crews start now, each with its chief. Tell the owner to open a crew and \
             talk to its chief: you do not do the crews' work."
        ),
    })
}

/// Whether a plan of the Maker still waits for the owner.
fn waiting_build(daemon: &Daemon, bot: &BotId) -> Result<bool, String> {
    let store = daemon.store();
    for id in store
        .pending_approvals(bot)
        .map_err(|err| err.to_string())?
    {
        let record = store.approval(&id).map_err(|err| err.to_string())?;
        if record.is_some_and(|record| record.approval.tool_name == BUILD_TOOL) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Checks the plan as the owner changed it, before the request settles: a
/// problem keeps the card open (spec 36.4).
pub(crate) fn check_changed(daemon: &Daemon, changed: &str) -> ApiResult<String> {
    let plan: Plan = serde_json::from_str(changed).map_err(|err| {
        ApiError::validation(format!("the changed plan could not be read: {err}"))
    })?;
    plan::check(daemon, &plan).map_err(ApiError::validation)?;
    serde_json::to_string(&plan).map_err(|err| ApiError::validation(err.to_string()))
}
