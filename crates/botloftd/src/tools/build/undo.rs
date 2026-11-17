//! Building the steps of a plan on crews that already exist (spec 36.8),
//! and putting them back. Each change saves how to undo it; a build that
//! fails halfway, and the owner's Undo, replay that list backwards.

use std::collections::HashSet;

use botloft_core::ids::{BotId, RoutineId};
use botloft_core::protocol::{
    BotIdParams, Missed, Overlap, Routine, RoutineIdParams, RoutinesSetEnabledParams,
    RoutinesUpdateParams, Schedule, VersionAuthor,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::warn;

use super::existing::{agent_of, crew_named, next_profile, next_routine, routine_of};
use super::plan::{self, Plan, Step};
use crate::service::{ApiResult, archive, bots, routines};
use crate::state::Daemon;
use crate::tools::bot_change::{self, Profile};
use crate::tools::calls::explain;
use crate::versions::Origin;

/// How to put back one change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "undo", rename_all = "snake_case")]
pub(crate) enum Undo {
    /// A bot the build added to an existing crew.
    ArchiveBot { bot_id: BotId },
    /// A bot the build archived.
    UnarchiveBot { bot_id: BotId },
    /// A bot the build changed: how it was.
    Profile { bot_id: BotId, profile: Profile },
    /// A routine the build added to an existing crew.
    ArchiveRoutine { routine_id: RoutineId },
    /// A routine the build deleted.
    UnarchiveRoutine { routine_id: RoutineId },
    /// A routine the build changed: how it was.
    Routine {
        routine_id: RoutineId,
        name: String,
        prompt: String,
        schedule: Schedule,
        timezone: String,
        overlap: Overlap,
        missed: Missed,
        enabled: bool,
    },
}

impl Undo {
    fn routine(routine: &Routine) -> Self {
        Self::Routine {
            routine_id: routine.id.clone(),
            name: routine.name.clone(),
            prompt: routine.prompt.clone(),
            schedule: routine.schedule.clone(),
            timezone: routine.timezone.clone(),
            overlap: routine.overlap,
            missed: routine.missed,
            enabled: routine.enabled,
        }
    }

    fn run(&self, daemon: &Daemon) -> ApiResult<()> {
        match self {
            Self::ArchiveBot { bot_id } => bots::archive(
                daemon,
                BotIdParams {
                    bot_id: bot_id.clone(),
                },
            )
            .map(drop),
            Self::UnarchiveBot { bot_id } => archive::unarchive_bot(
                daemon,
                BotIdParams {
                    bot_id: bot_id.clone(),
                },
            )
            .map(drop),
            Self::Profile { bot_id, profile } => {
                let now = Profile::of(&bots::find(&daemon.store(), bot_id)?);
                bot_change::apply(
                    daemon,
                    bot_id,
                    &now,
                    profile,
                    Origin::by(VersionAuthor::Maker),
                )
                .map(drop)
            }
            Self::ArchiveRoutine { routine_id } => routines::archive(
                daemon,
                RoutineIdParams {
                    routine_id: routine_id.clone(),
                },
            )
            .map(drop),
            Self::UnarchiveRoutine { routine_id } => {
                routines::unarchive(daemon, routine_id).map(drop)
            }
            Self::Routine {
                routine_id,
                name,
                prompt,
                schedule,
                timezone,
                overlap,
                missed,
                enabled,
            } => {
                routines::update(
                    daemon,
                    RoutinesUpdateParams {
                        routine_id: routine_id.clone(),
                        name: Some(name.clone()),
                        prompt: Some(prompt.clone()),
                        schedule: Some(schedule.clone()),
                        timezone: Some(timezone.clone()),
                        overlap: Some(*overlap),
                        missed: Some(*missed),
                    },
                )?;
                set_enabled(daemon, routine_id, *enabled)
            }
        }
    }
}

fn set_enabled(daemon: &Daemon, routine: &RoutineId, enabled: bool) -> ApiResult<()> {
    routines::set_enabled(
        daemon,
        RoutinesSetEnabledParams {
            routine_id: routine.clone(),
            enabled,
        },
    )
    .map(drop)
}

/// Puts back the changes, newest first; a change that cannot go back (the
/// owner deleted the bot since) is skipped.
pub(crate) fn replay(daemon: &Daemon, journal: &[Undo]) {
    for undo in journal.iter().rev() {
        if let Err(err) = undo.run(daemon) {
            warn!("could not undo a change of the Maker: {err}");
        }
    }
}

/// Undoes the changes a build saved as JSON (`builds.undo`).
pub(crate) fn undo_changes(daemon: &Daemon, saved: &str) {
    match serde_json::from_str::<Vec<Undo>>(saved) {
        Ok(journal) => replay(daemon, &journal),
        Err(err) => warn!("could not read what a build changed: {err}"),
    }
}

/// Builds the steps on existing crews (those whose `crew` is not a `ref`
/// of the plan): first the new bots, then the changed ones, then the
/// routines, then the archived bots. Returns a line per change for the Maker.
pub(crate) fn apply(
    daemon: &Daemon,
    plan: &Plan,
    refs: &HashSet<String>,
    journal: &mut Vec<Undo>,
) -> Result<Vec<Value>, String> {
    let ours = |crew: &str| !refs.contains(crew);
    let mut report = Vec::new();
    for step in &plan.steps {
        if let Step::AddAgent(agent) = step
            && ours(&agent.crew)
        {
            let crew = crew_named(daemon, &agent.crew)?;
            let (bot, handle) = super::apply::add_bot(daemon, &crew, agent)?;
            journal.push(Undo::ArchiveBot { bot_id: bot });
            report.push(json!({ "crew": crew.name, "added": format!("@{handle}") }));
        }
    }
    for step in &plan.steps {
        if let Step::ChangeAgent(change) = step {
            let crew = crew_named(daemon, &change.crew)?;
            let record = agent_of(daemon, &crew, &change.agent)?;
            let before = Profile::of(&record);
            let next = next_profile(daemon, &record, change)?;
            let bot = bot_change::apply(
                daemon,
                &record.id,
                &before,
                &next,
                Origin::by(VersionAuthor::Maker),
            )
            .map_err(|err| format!("{}: {}", change.agent, explain(err)))?;
            journal.push(Undo::Profile {
                bot_id: record.id,
                profile: before,
            });
            report.push(json!({ "crew": crew.name, "changed": format!("@{}", bot.handle) }));
        }
    }
    for step in &plan.steps {
        match step {
            Step::AddRoutine(new) if ours(&new.crew) => {
                let crew = crew_named(daemon, &new.crew)?;
                let bot = agent_of(daemon, &crew, &new.agent)?;
                let made = routines::create(daemon, plan::routine_params(new, bot.id))
                    .map_err(|err| format!("{}: {}", new.name, explain(err)))?;
                journal.push(Undo::ArchiveRoutine {
                    routine_id: made.id,
                });
                report.push(json!({ "crew": crew.name, "routine_added": new.name }));
            }
            Step::ChangeRoutine(change) => {
                let crew = crew_named(daemon, &change.crew)?;
                let routine = routine_of(daemon, &crew, change.agent.as_deref(), &change.routine)?;
                let next = next_routine(&routine, change);
                let fail = |err| format!("{}: {}", change.routine, explain(err));
                journal.push(Undo::routine(&routine));
                routines::update(
                    daemon,
                    RoutinesUpdateParams {
                        routine_id: routine.id.clone(),
                        name: Some(next.name.clone()),
                        prompt: Some(next.prompt),
                        schedule: Some(next.schedule),
                        timezone: Some(next.timezone),
                        overlap: None,
                        missed: None,
                    },
                )
                .map_err(fail)?;
                if let Some(enabled) = change.enabled {
                    set_enabled(daemon, &routine.id, enabled).map_err(fail)?;
                }
                report.push(json!({ "crew": crew.name, "routine_changed": next.name }));
            }
            Step::DeleteRoutine(delete) => {
                let crew = crew_named(daemon, &delete.crew)?;
                let routine = routine_of(daemon, &crew, delete.agent.as_deref(), &delete.routine)?;
                routines::archive(
                    daemon,
                    RoutineIdParams {
                        routine_id: routine.id.clone(),
                    },
                )
                .map_err(|err| format!("{}: {}", delete.routine, explain(err)))?;
                journal.push(Undo::UnarchiveRoutine {
                    routine_id: routine.id,
                });
                report.push(json!({ "crew": crew.name, "routine_deleted": routine.name }));
            }
            _ => {}
        }
    }
    // Last, so the routines of a bot it archives can still be changed.
    for step in &plan.steps {
        if let Step::ArchiveAgent(gone) = step {
            let crew = crew_named(daemon, &gone.crew)?;
            let record = agent_of(daemon, &crew, &gone.agent)?;
            bots::archive(
                daemon,
                BotIdParams {
                    bot_id: record.id.clone(),
                },
            )
            .map_err(|err| format!("{}: {}", gone.agent, explain(err)))?;
            journal.push(Undo::UnarchiveBot { bot_id: record.id });
            report.push(json!({ "crew": crew.name, "archived": format!("@{}", record.handle) }));
        }
    }
    Ok(report)
}
