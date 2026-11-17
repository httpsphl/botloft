//! Steps of a Maker's plan on crews that already exist (spec 36.8): change
//! a bot (instructions included), archive one, change or delete a routine.
//! `add_agent` and `add_routine` also take an existing crew by its name.
//! The daemon fills in `before` for the owner's card; building reads the
//! crew again and never trusts it.

use botloft_core::protocol::{Routine, RoutinesCreateParams, Schedule};
use botloft_store::BotRecord;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub(crate) use super::lookup::{agent_of, crew_named, handles, routine_of};
use super::plan::{Plan, STEP_WHY_MAX, Step};
use crate::service::{lead, routines};
use crate::state::Daemon;
use crate::tools::bot_change::{Change, Profile, changed_profile};
use crate::tools::calls::explain;
use crate::tools::routine::PROMPT_MAX;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChangeAgent {
    pub crew: String,
    /// The bot's handle, or `chief`.
    pub agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The whole new instructions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Added after the bot's instructions, under "From the Maker:".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub add_instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArchiveAgent {
    pub crew: String,
    pub agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChangeRoutine {
    pub crew: String,
    /// The routine's name (or id); `agent` picks between two of one name.
    pub routine: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<Schedule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeleteRoutine {
    pub crew: String,
    pub routine: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
}

/// The bot after a `change_agent`, checked as `bots.update` would.
pub(crate) fn next_profile(
    daemon: &Daemon,
    record: &BotRecord,
    step: &ChangeAgent,
) -> Result<Profile, String> {
    if step.instructions.is_some() && step.add_instructions.is_some() {
        return Err(format!(
            "{}: give either instructions (all of them) or add_instructions, not both",
            step.agent
        ));
    }
    let joined = step.add_instructions.as_deref().map(|extra| {
        format!(
            "{}\n\nFrom the Maker:\n{}",
            record.instructions.trim_end(),
            extra.trim()
        )
    });
    let change = Change {
        name: step.name.as_deref(),
        role: step.role.as_deref(),
        instructions: step.instructions.as_deref().or(joined.as_deref()),
        model: step.model.as_deref(),
        effort: step.effort.as_deref(),
    };
    changed_profile(daemon, record, &change)
        .map_err(|err| format!("{}: {}", step.agent, explain(err)))
}

/// The routine after a `change_routine`, as `routines.create` takes it.
pub(crate) fn next_routine(routine: &Routine, step: &ChangeRoutine) -> RoutinesCreateParams {
    RoutinesCreateParams {
        bot_id: routine.bot_id.clone(),
        name: step.name.clone().unwrap_or_else(|| routine.name.clone()),
        prompt: step
            .prompt
            .clone()
            .unwrap_or_else(|| routine.prompt.clone()),
        schedule: step
            .schedule
            .clone()
            .unwrap_or_else(|| routine.schedule.clone()),
        timezone: step
            .timezone
            .clone()
            .unwrap_or_else(|| routine.timezone.clone()),
        overlap: Some(routine.overlap),
        missed: Some(routine.missed),
    }
}

fn shown_routine(routine: &Routine) -> Value {
    json!({
        "name": routine.name,
        "prompt": routine.prompt,
        "schedule": routine.schedule,
        "timezone": routine.timezone,
        "enabled": routine.enabled,
    })
}

/// Fills in `before` for the owner's card.
pub(crate) fn resolve(daemon: &Daemon, step: &mut Step) -> Result<(), String> {
    match step {
        Step::ChangeAgent(change) => {
            let crew = crew_named(daemon, &change.crew)?;
            let record = agent_of(daemon, &crew, &change.agent)?;
            change.before = Some(json!(Profile::of(&record)));
        }
        Step::ArchiveAgent(archive) => {
            let crew = crew_named(daemon, &archive.crew)?;
            let record = agent_of(daemon, &crew, &archive.agent)?;
            archive.before = Some(json!({ "name": record.name, "role": record.role }));
        }
        Step::ChangeRoutine(change) => {
            let crew = crew_named(daemon, &change.crew)?;
            let routine = routine_of(daemon, &crew, change.agent.as_deref(), &change.routine)?;
            change.before = Some(shown_routine(&routine));
        }
        Step::DeleteRoutine(delete) => {
            let crew = crew_named(daemon, &delete.crew)?;
            let routine = routine_of(daemon, &crew, delete.agent.as_deref(), &delete.routine)?;
            delete.before = Some(shown_routine(&routine));
        }
        _ => {}
    }
    Ok(())
}

fn short(what: &str, why: Option<&String>) -> Result<(), String> {
    if why.is_some_and(|why| why.trim().chars().count() > STEP_WHY_MAX) {
        return Err(format!(
            "{what}: why must have at most {STEP_WHY_MAX} characters"
        ));
    }
    Ok(())
}

/// Checks the steps on existing crews; nothing impossible reaches the owner.
pub(crate) fn check(daemon: &Daemon, plan: &Plan) -> Result<(), String> {
    for step in &plan.steps {
        match step {
            Step::ChangeAgent(change) => {
                short(&change.agent, change.why.as_ref())?;
                let crew = crew_named(daemon, &change.crew)?;
                let record = agent_of(daemon, &crew, &change.agent)?;
                if next_profile(daemon, &record, change)? == Profile::of(&record) {
                    return Err(format!("{}: that is how the bot is already", change.agent));
                }
            }
            Step::ArchiveAgent(archive) => {
                short(&archive.agent, archive.why.as_ref())?;
                let crew = crew_named(daemon, &archive.crew)?;
                let record = agent_of(daemon, &crew, &archive.agent)?;
                if lead::is_lead(&crew, &record.id) {
                    return Err(format!(
                        "{}: the chief of a crew cannot be archived; change it instead",
                        archive.agent
                    ));
                }
            }
            Step::ChangeRoutine(change) => {
                short(&change.routine, change.why.as_ref())?;
                let crew = crew_named(daemon, &change.crew)?;
                let routine = routine_of(daemon, &crew, change.agent.as_deref(), &change.routine)?;
                let next = next_routine(&routine, change);
                if next.prompt.chars().count() > PROMPT_MAX {
                    return Err(format!("prompt must be at most {PROMPT_MAX} characters"));
                }
                routines::check(daemon, &next)
                    .map_err(|err| format!("{}: {}", change.routine, explain(err)))?;
                let same = next.name == routine.name
                    && next.prompt == routine.prompt
                    && next.schedule == routine.schedule
                    && next.timezone == routine.timezone
                    && change.enabled.is_none_or(|on| on == routine.enabled);
                if same {
                    return Err(format!(
                        "{}: that is how the routine is already",
                        change.routine
                    ));
                }
            }
            Step::DeleteRoutine(delete) => {
                short(&delete.routine, delete.why.as_ref())?;
                let crew = crew_named(daemon, &delete.crew)?;
                routine_of(daemon, &crew, delete.agent.as_deref(), &delete.routine)?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Whether the plan archives or deletes something: then the owner is always
/// asked, even when the Maker may build without asking (spec 36.4).
pub(crate) fn takes_away(plan: &Plan) -> bool {
    plan.steps
        .iter()
        .any(|step| matches!(step, Step::ArchiveAgent(_) | Step::DeleteRoutine(_)))
}
