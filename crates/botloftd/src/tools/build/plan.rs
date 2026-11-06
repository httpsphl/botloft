//! A plan of the Maker (spec 36.4): what `build` takes, what the owner's card
//! shows and changes, and the checks that keep anything impossible from
//! reaching the owner.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use botloft_core::ids::BotId;
use botloft_core::protocol::{AgentKind, BotEffort, BotModel, RoutinesCreateParams, Schedule};
use botloft_core::{slug, validate};
use serde::{Deserialize, Serialize};

use super::existing;
use crate::catalog;
use crate::service::bots::NewBot;
use crate::service::routines;
use crate::state::Daemon;
use crate::tools::calls::explain;

pub(super) const TITLE_MAX: usize = 120;
pub(super) const WHY_MAX: usize = 600;
pub(super) const STEP_WHY_MAX: usize = 300;
pub(super) const MEMORY_MAX: usize = 6_000;
pub(super) const STEPS_MAX: usize = 40;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Plan {
    pub title: String,
    pub why: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub(crate) enum Step {
    CreateCrew(NewCrew),
    AddAgent(NewAgent),
    AddRoutine(NewRoutine),
    ChangeAgent(existing::ChangeAgent),
    ArchiveAgent(existing::ArchiveAgent),
    ChangeRoutine(existing::ChangeRoutine),
    DeleteRoutine(existing::DeleteRoutine),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NewCrew {
    /// How the other steps name the crew before it exists.
    #[serde(rename = "ref")]
    pub key: String,
    pub name: String,
    pub goal: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lead_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lead_role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lead_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lead_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    /// Chosen by the owner in the card; the crew's `shared` folder without.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_folder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NewAgent {
    pub crew: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub instructions: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NewRoutine {
    pub crew: String,
    /// A bot's name in the plan, or `chief`.
    pub agent: String,
    pub name: String,
    pub prompt: String,
    pub schedule: Schedule,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
}

impl NewAgent {
    /// The bot for gent, the owner's default, as catalog.add makes one.
    pub(crate) fn new_bot(&self, agent: AgentKind) -> Result<NewBot, String> {
        let model = self
            .model
            .as_deref()
            .map(|model| {
                model.parse::<BotModel>().map_err(|_| {
                    format!(
                        "{}: model must be \"default\", \"fable\", \"opus\", \"sonnet\" or \"haiku\"",
                        self.name
                    )
                })
            })
            .transpose()?;
        let mut new = NewBot::check(
            &self.name,
            &self.role,
            &self.instructions,
            None,
            model,
            Some(agent),
        )
        .map_err(|err| format!("{}: {}", self.name, explain(err)))?;
        new.effort = effort(self.effort.as_deref())?;
        Ok(new)
    }
}

pub(crate) fn effort(effort: Option<&str>) -> Result<BotEffort, String> {
    effort.map_or(Ok(BotEffort::Default), |effort| {
        effort.parse().map_err(|_| {
            "effort must be \"default\", \"low\", \"medium\", \"high\", \"xhigh\" or \"max\""
                .to_owned()
        })
    })
}

pub(crate) fn lead_model(crew: &NewCrew) -> Result<Option<BotModel>, String> {
    crew.lead_model
        .as_deref()
        .map(|model| {
            model
                .parse::<BotModel>()
                .map_err(|_| format!("{}: lead_model is not a model", crew.name))
        })
        .transpose()
}

/// Fills in what the daemon knows: the catalog's role, instructions, model
/// and effort for a `template`, the chief's words in the owner's language and
/// the computer's time zone. Afterwards the plan reads the same to the owner
/// whatever the Maker left out.
pub(crate) fn resolve(daemon: &Daemon, plan: &mut Plan) -> Result<(), String> {
    let (chief, chief_role) = chief_words(&daemon.desktop.locale());
    for step in &mut plan.steps {
        match step {
            Step::CreateCrew(crew) => {
                crew.lead_name.get_or_insert_with(|| chief.to_owned());
                crew.lead_role.get_or_insert_with(|| chief_role.to_owned());
            }
            Step::AddAgent(agent) => resolve_template(agent)?,
            Step::AddRoutine(routine) => {
                routine
                    .timezone
                    .get_or_insert_with(crate::tools::routine::system_zone);
            }
            other => existing::resolve(daemon, other)?,
        }
    }
    Ok(())
}

fn resolve_template(agent: &mut NewAgent) -> Result<(), String> {
    let Some(id) = agent.template.take() else {
        if agent.role.trim().is_empty() || agent.instructions.trim().is_empty() {
            return Err(format!(
                "{}: role and instructions are required unless you give a template",
                agent.name
            ));
        }
        return Ok(());
    };
    let sheet = catalog::get(id.trim())
        .ok_or_else(|| format!("no bot template named {id}; list them with list_bot_templates"))?;
    if agent.role.trim().is_empty() {
        agent.role = sheet.role;
    }
    let extra = agent.instructions.trim();
    agent.instructions = if extra.is_empty() {
        sheet.instructions
    } else {
        format!("{}\n\nFor this crew:\n{extra}", sheet.instructions)
    };
    agent
        .model
        .get_or_insert_with(|| sheet.model.as_str().to_owned());
    agent
        .effort
        .get_or_insert_with(|| sheet.effort.as_str().to_owned());
    Ok(())
}

/// The chief's name and role as the app writes them (spec 10.2).
fn chief_words(locale: &str) -> (&'static str, &'static str) {
    if locale.starts_with("pt") {
        (
            "Chefe",
            "Lidera a equipe: planeja o trabalho, sugere agentes novos e distribui as tarefas",
        )
    } else if locale.starts_with("es") {
        (
            "Jefe",
            "Dirige el equipo: planea el trabajo, sugiere agentes nuevos y reparte las tareas",
        )
    } else {
        (
            "Chief",
            "Leads the crew: plans the work, suggests new agents and hands out tasks",
        )
    }
}

fn within(what: &str, text: &str, max: usize, required: bool) -> Result<(), String> {
    let count = text.trim().chars().count();
    if (required && count == 0) || count > max {
        return Err(format!("{what} must have at most {max} characters"));
    }
    Ok(())
}

/// Checks a resolved plan, as the Maker sent it or as the owner changed it.
pub(crate) fn check(daemon: &Daemon, plan: &Plan) -> Result<(), String> {
    within("title", &plan.title, TITLE_MAX, true)?;
    within("why", &plan.why, WHY_MAX, true)?;
    if plan.steps.is_empty() || plan.steps.len() > STEPS_MAX {
        return Err(format!("a plan has from 1 to {STEPS_MAX} steps"));
    }
    // Each crew of the plan, by ref: the handles of its bots.
    let mut crews: HashMap<&str, HashSet<String>> = HashMap::new();
    for step in &plan.steps {
        let Step::CreateCrew(crew) = step else {
            continue;
        };
        validate::name("name", &crew.name).map_err(|err| err.to_string())?;
        validate::instructions(&crew.goal).map_err(|err| err.to_string())?;
        within(&format!("{}: goal", crew.name), &crew.goal, 8_000, true)?;
        within(
            "memory",
            crew.memory.as_deref().unwrap_or(""),
            MEMORY_MAX,
            false,
        )?;
        lead_model(crew)?;
        effort(crew.lead_effort.as_deref())?;
        let lead = crew.lead_name.as_deref().unwrap_or_default();
        NewBot::check(
            lead,
            crew.lead_role.as_deref().unwrap_or_default(),
            &crew.goal,
            None,
            None,
            None,
        )
        .map_err(|err| format!("{}: {}", crew.name, explain(err)))?;
        let handles = HashSet::from([slug::slugify(lead, "bot")]);
        if crews.insert(crew.key.as_str(), handles).is_some() {
            return Err(format!("two crews use the ref \"{}\"", crew.key));
        }
    }
    for step in &plan.steps {
        let Step::AddAgent(agent) = step else {
            continue;
        };
        if let Some(why) = &agent.why {
            within("why", why, STEP_WHY_MAX, false)?;
        }
        agent.new_bot(crate::service::bots::default_agent(daemon))?;
        let handles = match crews.entry(agent.crew.as_str()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(existing::handles(daemon, &agent.crew)?),
        };
        if !handles.insert(slug::slugify(&agent.name, "bot")) {
            return Err(format!("two bots of a crew are called {}", agent.name));
        }
        let max = daemon.bots.max_per_crew;
        if handles.len() > max {
            return Err(format!("a crew has at most {max} bots, its chief included"));
        }
    }
    for step in &plan.steps {
        let Step::AddRoutine(routine) = step else {
            continue;
        };
        let handles = match crews.entry(routine.crew.as_str()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(existing::handles(daemon, &routine.crew)?),
        };
        let agent = slug::slugify(&routine.agent, "bot");
        if routine.agent != "chief" && !handles.contains(&agent) {
            return Err(format!(
                "{}: no bot called {} in that crew",
                routine.name, routine.agent
            ));
        }
        within(
            "prompt",
            &routine.prompt,
            crate::tools::routine::PROMPT_MAX,
            true,
        )?;
        routines::check(daemon, &routine_params(routine, BotId::generate()))
            .map_err(|err| format!("{}: {}", routine.name, explain(err)))?;
    }
    existing::check(daemon, plan)
}

pub(crate) fn routine_params(routine: &NewRoutine, bot: BotId) -> RoutinesCreateParams {
    RoutinesCreateParams {
        bot_id: bot,
        name: routine.name.clone(),
        prompt: routine.prompt.clone(),
        schedule: routine.schedule.clone(),
        timezone: routine
            .timezone
            .clone()
            .unwrap_or_else(crate::tools::routine::system_zone),
        overlap: None,
        missed: None,
    }
}
