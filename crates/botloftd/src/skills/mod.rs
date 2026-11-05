//! The owner's skills in the agents (spec 38): the skills in the owner's
//! own Claude Code folder and the claude.ai ones kept there, which agents
//! may use each one, and the copy each agent gets in its `.claude\skills`.
//! Only agents on Claude Code get them.

mod copy;
mod meta;
mod rules;

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use botloft_core::ids::{BotId, MessageId};
use botloft_core::protocol::{
    Message, MessageKind, SenderKind, Skill, SkillList, SkillSource, SkillsShareParams,
};
use tracing::{info, warn};

use crate::service::bots::active;
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

/// Where the owner's skills are looked for; the owner's own folder unless
/// a test says otherwise.
#[derive(Default)]
pub struct Skills {
    root: Mutex<Option<PathBuf>>,
}

impl Skills {
    /// Looks for the owner's skills in `root` instead of
    /// `~/.claude/skills`.
    pub fn use_root(&self, root: PathBuf) {
        *self.root.lock().unwrap_or_else(|p| p.into_inner()) = Some(root);
    }

    fn root(&self) -> Option<PathBuf> {
        let set = self.root.lock().unwrap_or_else(|p| p.into_inner()).clone();
        set.or_else(|| std::env::home_dir().map(|home| home.join(".claude").join("skills")))
    }
}

/// A skill found in the owner's folders.
struct Found {
    dir: PathBuf,
    source: SkillSource,
}

/// A folder name Botloft accepts as a skill's: what Claude Code would load,
/// and safe to join to a path.
fn valid(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && !name.starts_with('.')
        && !name.eq_ignore_ascii_case("synced")
        && name
            .chars()
            .all(|ch| ch.is_alphanumeric() || matches!(ch, '-' | '_' | '.' | ' '))
}

/// The skills in `dir`: each folder with a `SKILL.md`.
fn skills_in(dir: &Path, source: SkillSource, into: &mut BTreeMap<String, Found>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        // The folder itself may be a link the owner made; it is followed.
        if valid(&name) && path.join("SKILL.md").is_file() {
            into.entry(name).or_insert(Found { dir: path, source });
        }
    }
}

/// The owner's skills by name: their own win over the claude.ai ones.
fn found(daemon: &Daemon) -> BTreeMap<String, Found> {
    let mut all = BTreeMap::new();
    let Some(root) = daemon.skills.root() else {
        return all;
    };
    skills_in(&root, SkillSource::Personal, &mut all);
    if let Ok(accounts) = std::fs::read_dir(root.join("synced")) {
        for account in accounts.filter_map(Result::ok) {
            skills_in(&account.path(), SkillSource::ClaudeAi, &mut all);
        }
    }
    all
}

/// Botloft's own copies of the skills it gave (spec 38.3).
fn library(daemon: &Daemon) -> PathBuf {
    daemon.paths.home.join("skills")
}

/// `skills.list`: the owner's skills, and the ones only Botloft's copy is
/// left of that an agent still has.
pub fn list(daemon: &Daemon) -> ApiResult<SkillList> {
    let found = found(daemon);
    let store = daemon.store();
    let live: Vec<BotId> = store
        .bots(None, false)?
        .into_iter()
        .map(|bot| bot.id)
        .collect();
    let mut skills = Vec::new();
    let mut names: Vec<String> = found.keys().cloned().collect();
    for name in store.given_skills()? {
        if !found.contains_key(&name) && valid(&name) && library(daemon).join(&name).is_dir() {
            names.push(name);
        }
    }
    names.sort();
    for name in names {
        let (dir, source) = match found.get(&name) {
            Some(found) => (found.dir.clone(), found.source),
            None => (library(daemon).join(&name), SkillSource::Kept),
        };
        let Some(meta) = meta::read(&dir) else {
            continue;
        };
        let bot_ids = store
            .skill_bots(&name)?
            .into_iter()
            .filter(|bot| live.contains(bot))
            .collect();
        skills.push(Skill {
            name,
            description: meta.description,
            source,
            allowed_tools: meta.allowed_tools,
            runs_commands: meta.runs_commands,
            has_files: meta.has_files,
            bot_ids,
        });
    }
    Ok(SkillList { skills })
}

/// `skills.share`: the agents that may use the skill, in place of those
/// before. Each agent that gained or lost it starts again, with
/// `--resume`, once nothing is under way.
pub fn share(daemon: &Daemon, params: SkillsShareParams) -> ApiResult<Skill> {
    let name = params.name.trim().to_owned();
    let source = found(daemon).remove(&name).map(|found| found.dir);
    if !valid(&name) || (source.is_none() && !library(daemon).join(&name).is_dir()) {
        return Err(ApiError::NotFound(format!("skill {name}")));
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
    if let Some(source) = &source {
        let copy = library(daemon).join(&name);
        std::fs::create_dir_all(library(daemon)).map_err(ApiError::Workspace)?;
        copy::copy_skill(source, &copy).map_err(ApiError::Workspace)?;
    }
    let had = daemon.store().skill_bots(&name)?;
    daemon.store().set_skill_bots(&name, &wanted)?;
    for bot in had.iter().chain(&wanted) {
        if had.contains(bot) != wanted.contains(bot) {
            daemon.supervisor.launch_settings_changed(bot);
        }
    }
    info!("the owner chose which agents use a skill");
    if let Some(skill) = list(daemon)?
        .skills
        .into_iter()
        .find(|skill| skill.name == name)
    {
        return Ok(skill);
    }
    // The original is gone and no agent has it any more: Botloft's copy
    // goes too.
    let copy = library(daemon).join(&name);
    let meta = meta::read(&copy).unwrap_or_default();
    if copy::is_ours(&copy) {
        std::fs::remove_dir_all(&copy).map_err(ApiError::Workspace)?;
    }
    Ok(Skill {
        name,
        description: meta.description,
        source: SkillSource::Kept,
        allowed_tools: meta.allowed_tools,
        runs_commands: meta.runs_commands,
        has_files: meta.has_files,
        bot_ids: Vec::new(),
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
    let chosen = daemon.store().bot_skills(bot).map_err(io::Error::other)?;
    let found = found(daemon);
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
        .into_keys()
        .filter(|name| !chosen.contains(name))
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

pub(crate) fn given(daemon: &Daemon, bot: &BotId, name: &str) -> bool {
    daemon
        .store()
        .bot_skills(bot)
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
