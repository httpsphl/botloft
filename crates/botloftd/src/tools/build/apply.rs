//! Building a checked plan (spec 36.4): the crews with their chiefs and
//! memory, then the bots, then the routines, then the steps on crews that
//! already exist (`undo.rs`). When a step fails, the changes are put back
//! and the crews already made are deleted again, so a plan is built whole
//! or not at all.

use std::collections::{HashMap, HashSet};

use botloft_core::ids::{ApprovalId, BotId, BuildId, CrewId};
use botloft_core::protocol::{
    BUILD_UNDO_DAYS, BotEffort, BotsSetEffortParams, Build, Crew, CrewsCreateParams,
    CrewsDeleteParams, NewLead,
};
use serde_json::{Value, json};
use tracing::{info, warn};

use super::plan::{self, Plan, Step};
use super::undo::{self, Undo};
use crate::service::{ApiResult, bots, crews, delete, models, routines};
use crate::state::Daemon;
use crate::tools::calls::explain;

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// A crew of the plan once made: the bots by handle, for the routines.
struct Made {
    crew: Crew,
    chief: BotId,
    bots: HashMap<String, BotId>,
    report: Value,
}

/// Builds `plan` for the Maker `maker` and saves what it made.
pub(crate) fn build(
    daemon: &Daemon,
    maker: &BotId,
    approval: Option<ApprovalId>,
    plan: &Plan,
) -> Result<(Build, Value), String> {
    let mut made: Vec<(String, Made)> = Vec::new();
    let mut journal: Vec<Undo> = Vec::new();
    let refs: HashSet<String> = plan
        .steps
        .iter()
        .filter_map(|step| match step {
            Step::CreateCrew(crew) => Some(crew.key.clone()),
            _ => None,
        })
        .collect();
    let changes = match make(daemon, plan, &mut made)
        .and_then(|()| undo::apply(daemon, plan, &refs, &mut journal))
    {
        Ok(changes) => changes,
        Err(err) => {
            undo::replay(daemon, &journal);
            undo_partly(daemon, &made);
            return Err(err);
        }
    };
    let now = daemon.clock.now_ms();
    let build = Build {
        id: BuildId::generate(),
        bot_id: maker.clone(),
        approval_id: approval,
        title: plan.title.clone(),
        crew_ids: made.iter().map(|(_, made)| made.crew.id.clone()).collect(),
        applied_at: now,
        undo_until: now + BUILD_UNDO_DAYS * DAY_MS,
        undone_at: None,
    };
    let saved = serde_json::to_string(&journal).unwrap_or_else(|_| "[]".to_owned());
    let store = daemon.store();
    if let Err(err) = store
        .insert_build(&build)
        .and_then(|()| store.set_build_changes(&build.id, &saved))
    {
        warn!("could not save a build: {err}");
    }
    drop(store);
    info!(
        build = %build.id,
        crews = build.crew_ids.len(),
        changes = journal.len(),
        "the Maker built a plan"
    );
    let crews: Vec<Value> = made.into_iter().map(|(_, made)| made.report).collect();
    Ok((build, json!({ "crews": crews, "changes": changes })))
}

fn make(daemon: &Daemon, plan: &Plan, made: &mut Vec<(String, Made)>) -> Result<(), String> {
    for step in &plan.steps {
        if let Step::CreateCrew(new) = step {
            made.push((new.key.clone(), make_crew(daemon, new)?));
        }
    }
    // Steps on crews that already exist are built afterwards.
    let find = |made: &mut Vec<(String, Made)>, key: &str| {
        made.iter_mut().position(|(made_key, _)| made_key == key)
    };
    for step in &plan.steps {
        if let Step::AddAgent(agent) = step
            && let Some(at) = find(made, &agent.crew)
        {
            let crew = made[at].1.crew.clone();
            let bot = add_bot(daemon, &crew, agent)?;
            let entry = &mut made[at].1;
            if let Some(list) = entry.report["agents"].as_array_mut() {
                list.push(json!({"handle": bot.1, "name": agent.name}));
            }
            entry.bots.insert(bot.1, bot.0);
        }
    }
    for step in &plan.steps {
        if let Step::AddRoutine(routine) = step
            && let Some(at) = find(made, &routine.crew)
        {
            let entry = &mut made[at].1;
            let handle = botloft_core::slug::slugify(&routine.agent, "bot");
            let bot = match entry.bots.get(&handle) {
                Some(bot) => bot.clone(),
                None => entry.chief.clone(),
            };
            routines::create(daemon, plan::routine_params(routine, bot))
                .map_err(|err| format!("{}: {}", routine.name, explain(err)))?;
            if let Some(list) = entry.report["routines"].as_array_mut() {
                list.push(json!(routine.name));
            }
        }
    }
    Ok(())
}

fn make_crew(daemon: &Daemon, new: &plan::NewCrew) -> Result<Made, String> {
    let crew = crews::create(
        daemon,
        CrewsCreateParams {
            name: new.name.clone(),
            work_folder: new.work_folder.clone(),
            lead: Some(NewLead {
                name: new.lead_name.clone().unwrap_or_default(),
                role: new.lead_role.clone().unwrap_or_default(),
                instructions: new.goal.clone(),
                model: plan::lead_model(new)?,
            }),
            auto_mode: None,
        },
    )
    .map_err(|err| format!("{}: {}", new.name, explain(err)))?;
    let chief = crew
        .lead_bot_id
        .clone()
        .ok_or_else(|| format!("{}: the crew has no chief", new.name))?;
    set_effort(daemon, &chief, new.lead_effort.as_deref())?;
    if let Some(memory) = new.memory.as_deref().filter(|text| !text.trim().is_empty()) {
        let file = daemon.paths.crew_dir(&crew.slug).join("CLAUDE.md");
        std::fs::write(&file, format!("{}\n", memory.trim())).map_err(|err| {
            format!(
                "{}: the crew's memory could not be written: {err}",
                new.name
            )
        })?;
    }
    let handle = botloft_core::slug::slugify(new.lead_name.as_deref().unwrap_or_default(), "bot");
    let report = json!({
        "ref": new.key,
        "crew": crew.name,
        "crew_id": crew.id,
        "chief": format!("@{handle}"),
        "agents": [],
        "routines": [],
    });
    let mut bots = HashMap::new();
    bots.insert(handle, chief.clone());
    Ok(Made {
        crew,
        chief,
        bots,
        report,
    })
}

/// The bot, as `catalog.add` makes one: Manual mode (or the crew's
/// automatic one), no tools, logins, keys or reach into other crews.
pub(super) fn add_bot(
    daemon: &Daemon,
    crew: &Crew,
    agent: &plan::NewAgent,
) -> Result<(BotId, String), String> {
    let new = agent.new_bot(bots::default_agent(daemon))?;
    let effort = new.effort;
    let store = daemon.store();
    let record = bots::insert(daemon, &store, crew, BotId::generate(), new)
        .map_err(|err| format!("{}: {}", agent.name, explain(err)))?;
    let bot = bots::changed(daemon, &store, crew, record);
    drop(store);
    daemon.supervisor.wake();
    if effort != BotEffort::Default {
        set_effort(daemon, &bot.id, Some(effort.as_str()))?;
    }
    Ok((bot.id, bot.handle))
}

fn set_effort(daemon: &Daemon, bot: &BotId, effort: Option<&str>) -> Result<(), String> {
    let effort = plan::effort(effort)?;
    if effort == BotEffort::Default {
        return Ok(());
    }
    models::set_effort(
        daemon,
        BotsSetEffortParams {
            bot_id: bot.clone(),
            effort,
        },
    )
    .map(drop)
    .map_err(explain)
}

/// Deletes the crews a failed build made, with their folders.
fn undo_partly(daemon: &Daemon, made: &[(String, Made)]) {
    for (_, made) in made {
        if let Err(err) = delete_crew(daemon, &made.crew.id) {
            warn!(crew = %made.crew.id, "could not take back a crew of a failed build: {err}");
        }
    }
}

fn delete_crew(daemon: &Daemon, crew: &CrewId) -> ApiResult<()> {
    delete::crew(
        daemon,
        CrewsDeleteParams {
            crew_id: crew.clone(),
            recycle_folder: Some(true),
        },
    )
    .map(drop)
}
