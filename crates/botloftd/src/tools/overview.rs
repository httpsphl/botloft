//! `crews_overview` (spec 36.8): only for the Maker. The owner's crews as
//! they are set up: the bots with their role, model and effort, and the
//! routines. With `crew`, one crew in full, with each bot's instructions and
//! each routine's text, so the Maker can rewrite them. Never messages,
//! tasks or files.

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotsListParams, Crew, RoutinesListParams, Schedule};
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::explain;
use crate::service::{bots, lead, maker, routines};
use crate::state::Daemon;

pub const CREWS_OVERVIEW: &str = "crews_overview";

/// How much of a chief's goal the summary shows.
const GOAL_CHARS: usize = 600;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Args {
    #[serde(default)]
    crew: Option<String>,
}

pub(super) fn tool() -> Value {
    json!({
        "name": CREWS_OVERVIEW,
        "title": "The owner's crews",
        "description": "Only for the Maker. The owner's crews as they are set up: each crew's \
            chief and goal, its bots (handle, name, role, model, effort, state) and its \
            routines (name, bot, when, on or off). Give `crew` (a crew's name) to see that crew \
            in full, with each bot's instructions and each routine's text, before you change \
            them with build. It never shows messages, tasks or files.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "crew": { "type": "string", "description": "A crew's name, to see it in full." },
            },
            "additionalProperties": false,
        },
        "annotations": { "readOnlyHint": true },
    })
}

pub(super) fn call(daemon: &Daemon, bot: &BotId, args: Args) -> Result<Value, String> {
    if !maker::is_maker(&daemon.store(), bot).map_err(explain)? {
        return Err("only the Maker sees all the crews".to_owned());
    }
    let crews: Vec<Crew> = daemon
        .store()
        .crews(false)
        .map_err(|err| explain(err.into()))?
        .into_iter()
        .filter(|crew| !maker::is_maker_crew(crew))
        .collect();
    let Some(wanted) = args.crew else {
        let list = crews
            .iter()
            .map(|crew| shown(daemon, crew, false))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(json!({ "crews": list }));
    };
    let key = wanted.trim().to_lowercase();
    let crew = crews
        .iter()
        .find(|crew| crew.name.trim().to_lowercase() == key)
        .ok_or_else(|| format!("no crew is called {wanted}; leave out `crew` to list them"))?;
    shown(daemon, crew, true)
}

fn when(schedule: &Schedule) -> Value {
    serde_json::to_value(schedule).unwrap_or(Value::Null)
}

fn shown(daemon: &Daemon, crew: &Crew, full: bool) -> Result<Value, String> {
    let list = bots::list(
        daemon,
        BotsListParams {
            crew_id: Some(crew.id.clone()),
        },
    )
    .map_err(explain)?;
    let records = daemon
        .store()
        .bots(Some(&crew.id), false)
        .map_err(|err| explain(err.into()))?;
    let mut agents = Vec::new();
    let mut routine_list = Vec::new();
    let mut goal = String::new();
    for bot in &list {
        let chief = lead::is_lead(crew, &bot.id);
        let instructions = records
            .iter()
            .find(|record| record.id == bot.id)
            .map(|record| record.instructions.clone())
            .unwrap_or_default();
        if chief {
            goal = instructions.chars().take(GOAL_CHARS).collect();
        }
        let mut agent = json!({
            "handle": bot.handle,
            "name": bot.name,
            "role": bot.role,
            "chief": chief,
            "model": bot.model,
            "effort": bot.effort,
            "state": bot.state,
        });
        if full {
            agent["instructions"] = json!(instructions);
        }
        agents.push(agent);
        let own = routines::list(
            daemon,
            RoutinesListParams {
                bot_id: Some(bot.id.clone()),
            },
        )
        .map_err(explain)?;
        for routine in own {
            let mut item = json!({
                "name": routine.name,
                "agent": bot.handle,
                "enabled": routine.enabled,
                "schedule": when(&routine.schedule),
                "timezone": routine.timezone,
                "next_run": routines::next_run_text(&routine),
            });
            if full {
                item["prompt"] = json!(routine.prompt);
            }
            routine_list.push(item);
        }
    }
    Ok(json!({
        "crew": crew.name,
        "paused": crew.paused,
        "goal": goal,
        "agents": agents,
        "routines": routine_list,
    }))
}
