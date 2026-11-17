//! The owner's skills in the agents (spec 38): the skills in the owner's
//! own Claude Code folder and the claude.ai ones kept there, which agents
//! may use each one (chosen one by one, or every agent), and the copy each
//! agent gets in its `.claude\skills`. Only agents on Claude Code get them.

mod copy;
mod found;
mod give;
mod meta;
mod rules;
mod slash;

pub use found::Skills;
pub use give::{everyone, share};
pub use slash::commands;
pub(crate) use slash::seen;

use found::{Found, found, valid};

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use botloft_core::ids::{BotId, MessageId};
use botloft_core::protocol::{
    AgentKind, Message, MessageKind, SenderKind, Skill, SkillList, SkillSource, SkillsShareParams,
};
use botloft_store::BotRecord;
use tracing::warn;

use crate::service::ApiResult;
use crate::service::bots::active;
use crate::state::Daemon;

/// Botloft's own copies of the skills it gave (spec 38.3).
fn library(daemon: &Daemon) -> PathBuf {
    daemon.paths.home.join("skills")
}

/// The skills an agent can get, by name: the owner's that fit in a copy,
/// and the ones only Botloft's copy is left of that an agent still has.
fn names(daemon: &Daemon, found: &BTreeMap<String, Found>) -> botloft_store::Result<Vec<String>> {
    let (given, everyone) = {
        let store = daemon.store();
        (store.given_skills()?, store.everyone_skills()?)
    };
    let mut names: Vec<String> = found
        .iter()
        .filter(|(_, found)| found.fits)
        .map(|(name, _)| name.clone())
        .collect();
    for name in given.into_iter().chain(everyone) {
        if !found.contains_key(&name)
            && !names.contains(&name)
            && valid(&name)
            && library(daemon).join(&name).is_dir()
        {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

/// The skills an agent on Claude Code uses, out of `names` (spec 38.3): the
/// ones chosen for it, the ones for every agent, and all of them when the
/// owner gave every skill to every agent.
fn chosen(daemon: &Daemon, bot: &BotId, names: &[String]) -> botloft_store::Result<Vec<String>> {
    let store = daemon.store();
    if store.all_skills_everyone()? {
        return Ok(names.to_vec());
    }
    let mut chosen = store.bot_skills(bot)?;
    chosen.extend(store.everyone_skills()?);
    chosen.retain(|name| names.contains(name));
    chosen.sort();
    chosen.dedup();
    Ok(chosen)
}

fn on_claude_code(bot: &BotRecord) -> bool {
    bot.agent == AgentKind::Claude
}

/// `skills.list`: the owner's skills, and the ones only Botloft's copy is
/// left of that an agent still has.
pub fn list(daemon: &Daemon) -> ApiResult<SkillList> {
    let found = found(daemon);
    let mut names = names(daemon, &found)?;
    // Too big for a copy: listed, so the owner sees why, and given to no one.
    names.extend(
        found
            .iter()
            .filter(|(_, found)| !found.fits)
            .map(|(name, _)| name.clone()),
    );
    names.sort();
    let store = daemon.store();
    let all = store.all_skills_everyone()?;
    let everyone = store.everyone_skills()?;
    let claude: Vec<BotId> = store
        .bots(None, false)?
        .into_iter()
        .filter(on_claude_code)
        .map(|bot| bot.id)
        .collect();
    let mut skills = Vec::new();
    for name in names {
        let (dir, source) = match found.get(&name) {
            Some(found) => (found.dir.clone(), found.source),
            None => (library(daemon).join(&name), SkillSource::Kept),
        };
        let Some(meta) = meta::read(&dir) else {
            continue;
        };
        let for_everyone = everyone.contains(&name);
        let too_big = found.get(&name).is_some_and(|found| !found.fits);
        let bot_ids = if too_big {
            Vec::new()
        } else if all || for_everyone {
            claude.clone()
        } else {
            store
                .skill_bots(&name)?
                .into_iter()
                .filter(|bot| claude.contains(bot))
                .collect()
        };
        skills.push(Skill {
            name,
            description: meta.description,
            source,
            allowed_tools: meta.allowed_tools,
            runs_commands: meta.runs_commands,
            has_files: meta.has_files,
            bot_ids,
            everyone: for_everyone,
            too_big,
        });
    }
    Ok(SkillList {
        skills,
        everyone: all,
    })
}

/// Writes the agent's skills before it starts (spec 38.3): Botloft's copy
/// of each one it has, fresh from the owner's folder when the skill is
/// still there, in its `.claude\skills`, and its rules.
pub(crate) fn launch(daemon: &Daemon, bot: &BotId, workspace: &Path) -> io::Result<()> {
    if !crate::connectors::on_claude(daemon, bot) {
        copy::sync(&library(daemon), workspace, &[])?;
        return rules::write(workspace, &[], &[]);
    }
    let found = found(daemon);
    let chosen = names(daemon, &found)
        .and_then(|names| chosen(daemon, bot, &names))
        .map_err(io::Error::other)?;
    for name in &chosen {
        if let Some(found) = found.get(name) {
            let copy = library(daemon).join(name);
            std::fs::create_dir_all(library(daemon))?;
            if let Err(err) = copy::copy_skill(&found.dir, &copy) {
                warn!(bot = %bot, "could not copy a skill again: {err}");
            }
        }
    }
    copy::sync(&library(daemon), workspace, &chosen)?;
    let others: Vec<String> = found
        .into_iter()
        .filter(|(name, found)| found.fits && !chosen.contains(name))
        .map(|(name, _)| name)
        .collect();
    rules::write(workspace, &chosen, &others)
}

/// The owner's skill called `name`, by its folder's name, whatever the
/// case.
pub(crate) fn known(daemon: &Daemon, name: &str) -> Option<String> {
    let wanted = name.trim().to_lowercase();
    list(daemon)
        .ok()?
        .skills
        .into_iter()
        .map(|skill| skill.name)
        .find(|have| have.to_lowercase() == wanted)
}

/// Whether the owner's skill `name` is too big for a copy, so no agent can
/// have it.
pub(crate) fn too_big(daemon: &Daemon, name: &str) -> bool {
    found(daemon).get(name).is_some_and(|found| !found.fits)
}

/// Whether the agent uses the skill `name`, chosen for it or for every
/// agent.
pub(crate) fn given(daemon: &Daemon, bot: &BotId, name: &str) -> bool {
    names(daemon, &found(daemon))
        .and_then(|names| chosen(daemon, bot, &names))
        .is_ok_and(|names| names.iter().any(|have| have == name))
}

/// Gives the skill the agent asked for with `ask_skill` and queues a note
/// that tells it to go on once it has started again (spec 38.5).
pub(crate) fn grant(daemon: &Daemon, bot: &BotId, name: &str) -> ApiResult<()> {
    let mut bots = daemon.store().skill_bots(name)?;
    if !bots.contains(bot) {
        bots.push(bot.clone());
    }
    share(
        daemon,
        SkillsShareParams {
            name: name.to_owned(),
            bot_ids: bots,
        },
    )?;
    let store = daemon.store();
    let (crew, _) = active(&store, bot)?;
    let note = Message {
        id: MessageId::generate(),
        crew_id: crew.id,
        from_kind: SenderKind::System,
        from_bot_id: None,
        to_bot_id: bot.clone(),
        kind: MessageKind::System,
        body: format!(
            "The owner gave you the skill {name}. It is in .claude/skills now: go on with what \
             you were doing."
        ),
        task_id: None,
        routine_id: None,
        question_id: None,
        reply_to: None,
        attachments: Vec::new(),
        created_at: daemon.clock.now_ms(),
    };
    if let Err(err) = crate::service::messages::post(daemon, &store, note, None) {
        warn!(bot = %bot, "could not tell an agent about its new skill: {err}");
    }
    Ok(())
}
