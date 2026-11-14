//! Giving the owner's skills (spec 38.3): to the agents the owner chose,
//! or to every agent on Claude Code, the ones made later included. Each
//! agent whose skills changed starts again, with `--resume`, once nothing
//! is under way.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    Skill, SkillList, SkillSource, SkillsEveryoneParams, SkillsShareParams,
};
use tracing::info;

use super::{chosen, copy, found, library, list, meta, names, on_claude_code, valid};
use crate::service::bots::active;
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

/// `skills.share`: the agents that may use the skill, in place of those
/// before.
pub fn share(daemon: &Daemon, params: SkillsShareParams) -> ApiResult<Skill> {
    let name = params.name.trim().to_owned();
    let source = source(daemon, &name)?;
    if !params.bot_ids.is_empty() {
        refuse_too_big(daemon, &name)?;
    }
    let mut wanted: Vec<BotId> = Vec::new();
    {
        let store = daemon.store();
        for bot in params.bot_ids {
            active(&store, &bot)?;
            if !wanted.contains(&bot) {
                wanted.push(bot);
            }
        }
    }
    if let Some(bot) = wanted
        .iter()
        .find(|bot| !crate::connectors::on_claude(daemon, bot))
    {
        return Err(ApiError::Conflict(format!(
            "only agents on Claude Code can use skills, and {bot} is not one"
        )));
    }
    keep_copy(daemon, &name, source.as_deref())?;
    let before = uses(daemon)?;
    daemon.store().set_skill_bots(&name, &wanted)?;
    restart_changed(daemon, &before)?;
    info!("the owner chose which agents use a skill");
    let listed = list(daemon)?;
    let skill = match listed.skills.iter().find(|skill| skill.name == name) {
        Some(skill) => skill.clone(),
        // The original is gone and no agent has it any more: Botloft's
        // copy goes too, below.
        None => {
            let meta = meta::read(&library(daemon).join(&name)).unwrap_or_default();
            Skill {
                name,
                description: meta.description,
                source: SkillSource::Kept,
                allowed_tools: meta.allowed_tools,
                runs_commands: meta.runs_commands,
                has_files: meta.has_files,
                bot_ids: Vec::new(),
                everyone: false,
                too_big: false,
            }
        }
    };
    tidy(daemon, &listed)?;
    Ok(skill)
}

/// `skills.everyone`: one skill, or every skill, for every agent on Claude
/// Code, or not any more. Taking it back leaves each agent with the ones
/// chosen for it.
pub fn everyone(daemon: &Daemon, params: SkillsEveryoneParams) -> ApiResult<SkillList> {
    let before = uses(daemon)?;
    match params.name {
        Some(name) => {
            let name = name.trim().to_owned();
            let source = source(daemon, &name)?;
            if params.on {
                refuse_too_big(daemon, &name)?;
                keep_copy(daemon, &name, source.as_deref())?;
            }
            daemon.store().set_skill_everyone(&name, params.on)?;
            info!(
                on = params.on,
                "the owner changed whether every agent uses a skill"
            );
        }
        None => {
            daemon.store().set_all_skills_everyone(params.on)?;
            info!(
                on = params.on,
                "the owner changed whether every agent uses every skill"
            );
        }
    }
    restart_changed(daemon, &before)?;
    let listed = list(daemon)?;
    tidy(daemon, &listed)?;
    Ok(listed)
}

/// The owner's folder of the skill `name`, when it is still there; an error
/// when neither it nor Botloft's copy is.
fn source(daemon: &Daemon, name: &str) -> ApiResult<Option<PathBuf>> {
    let source = found(daemon).remove(name).map(|found| found.dir);
    if !valid(name) || (source.is_none() && !library(daemon).join(name).is_dir()) {
        return Err(ApiError::NotFound(format!("skill {name}")));
    }
    Ok(source)
}

/// A skill too big for a copy cannot be given (spec 38.3).
fn refuse_too_big(daemon: &Daemon, name: &str) -> ApiResult<()> {
    if super::too_big(daemon, name) {
        return Err(ApiError::Conflict(format!(
            "the skill {name} is too big to give: Botloft copies up to 2,000 files and 50 MB"
        )));
    }
    Ok(())
}

/// Botloft's copy, which the agents get and which stays if the owner
/// deletes the original.
fn keep_copy(daemon: &Daemon, name: &str, source: Option<&Path>) -> ApiResult<()> {
    if let Some(source) = source {
        std::fs::create_dir_all(library(daemon)).map_err(ApiError::Workspace)?;
        copy::copy_skill(source, &library(daemon).join(name)).map_err(ApiError::Workspace)?;
    }
    Ok(())
}

/// The skills each agent on Claude Code uses now.
fn uses(daemon: &Daemon) -> ApiResult<BTreeMap<BotId, Vec<String>>> {
    let names = names(daemon, &found(daemon))?;
    let bots: Vec<BotId> = daemon
        .store()
        .bots(None, false)?
        .into_iter()
        .filter(on_claude_code)
        .map(|bot| bot.id)
        .collect();
    let mut uses = BTreeMap::new();
    for bot in bots {
        let have = chosen(daemon, &bot, &names)?;
        uses.insert(bot, have);
    }
    Ok(uses)
}

/// Each agent whose skills are not the ones it had starts again.
fn restart_changed(daemon: &Daemon, before: &BTreeMap<BotId, Vec<String>>) -> ApiResult<()> {
    for (bot, now) in uses(daemon)? {
        if before.get(&bot).is_some_and(|had| *had != now) {
            daemon.supervisor.launch_settings_changed(&bot);
        }
    }
    Ok(())
}

/// Botloft's copies of the skills the owner no longer has and no agent
/// uses any more go.
fn tidy(daemon: &Daemon, listed: &SkillList) -> ApiResult<()> {
    let Ok(entries) = std::fs::read_dir(library(daemon)) else {
        return Ok(());
    };
    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        // A folder with a dot is a copy being made.
        if valid(&name)
            && copy::is_ours(&path)
            && !listed.skills.iter().any(|skill| skill.name == name)
        {
            std::fs::remove_dir_all(&path).map_err(ApiError::Workspace)?;
        }
    }
    Ok(())
}
