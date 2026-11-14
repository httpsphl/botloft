//! The menu of `/` in the chat (spec 38.11): the skills and plugin commands
//! an agent on Claude Code can run by name. What Botloft gave it comes from
//! Botloft, as it stands now; the rest (the skills that come with Claude
//! Code, the ones the agent made) from the last `system/init` of its Claude
//! Code. Claude Code's own commands (`/model`, `/clear`...) stay out.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use botloft_core::ids::BotId;
use botloft_core::protocol::{AgentKind, BotIdParams, SlashCommand, SlashCommandList, SlashSource};
use serde_json::Value;

use super::{copy, meta, valid};
use crate::service::ApiResult;
use crate::service::bots::active;
use crate::state::Daemon;

/// Longest description the menu gets, in characters.
const DESCRIPTION_MAX: usize = 200;
/// Longest name kept from what Claude Code lists.
const NAME_MAX: usize = 200;

/// Keeps the skills Claude Code lists in its `system/init` (with the ones
/// that come with it) and the slash commands of plugins (`plugin:name`).
pub(crate) fn seen(daemon: &Daemon, bot: &BotId, init: &Value) {
    let Some(skills) = init["skills"].as_array() else {
        return;
    };
    let plugin_commands = init["slash_commands"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|name| name.as_str().is_some_and(|name| name.contains(':')));
    let mut names: Vec<String> = Vec::new();
    for name in skills
        .iter()
        .chain(plugin_commands)
        .filter_map(Value::as_str)
    {
        if !name.is_empty() && name.len() <= NAME_MAX && !names.iter().any(|have| have == name) {
            names.push(name.to_owned());
        }
    }
    daemon.skills.remember(bot, names);
}

/// `bots.commands`: what the owner can type after `/` to the agent, by
/// name. Empty for an agent that is not on Claude Code.
pub fn commands(daemon: &Daemon, params: BotIdParams) -> ApiResult<SlashCommandList> {
    let bot = params.bot_id;
    let (agent, workspace) = {
        let store = daemon.store();
        let (crew, record) = active(&store, &bot)?;
        let workspace = daemon.paths.bot_workspace(&crew.slug, &record.slug);
        (record.agent, workspace)
    };
    if agent != AgentKind::Claude {
        return Ok(SlashCommandList {
            commands: Vec::new(),
        });
    }
    let mut chosen: BTreeMap<String, SlashSource> = BTreeMap::new();
    // What Botloft manages, given to this agent or not.
    let mut managed: Vec<String> = Vec::new();
    for skill in super::list(daemon)?.skills {
        if skill.bot_ids.contains(&bot) {
            chosen.insert(skill.name.clone(), SlashSource::Owner);
        }
        managed.push(skill.name);
    }
    for plugin in crate::plugins::list(daemon)?.plugins {
        let given = plugin.bot_ids.contains(&bot);
        for name in plugin.skills {
            if given {
                chosen.insert(name.clone(), SlashSource::Plugin);
            }
            managed.push(name);
        }
    }
    for name in daemon.skills.seen_by(&bot) {
        if managed.contains(&name) || chosen.contains_key(&name) {
            continue;
        }
        let from = if name.contains(':') {
            SlashSource::Plugin
        } else if own_skill(&workspace, &name) {
            SlashSource::Agent
        } else {
            SlashSource::ClaudeCode
        };
        chosen.insert(name, from);
    }
    let commands = chosen
        .into_iter()
        .map(|(name, from)| SlashCommand {
            description: describe(&workspace, &name),
            name,
            from,
        })
        .collect();
    Ok(SlashCommandList { commands })
}

/// A skill folder the agent made itself, not a copy Botloft put there.
fn own_skill(workspace: &Path, name: &str) -> bool {
    valid(name) && {
        let dir = workspace.join(".claude").join("skills").join(name);
        dir.join("SKILL.md").is_file() && !copy::is_ours(&dir)
    }
}

/// What the skill or command says it does, from its file in the agent's
/// folder; empty when there is none (the ones that come with Claude Code).
fn describe(workspace: &Path, name: &str) -> String {
    let claude = workspace.join(".claude");
    let files: Vec<PathBuf> = match name.split_once(':') {
        Some((plugin, item)) if valid(plugin) && valid(item) => {
            let plugin = claude.join("plugins").join(plugin);
            vec![
                plugin.join("skills").join(item).join("SKILL.md"),
                plugin.join("commands").join(format!("{item}.md")),
            ]
        }
        None if valid(name) => vec![claude.join("skills").join(name).join("SKILL.md")],
        _ => Vec::new(),
    };
    let text = files
        .iter()
        .find_map(|file| meta::description(file))
        .unwrap_or_default();
    if text.chars().count() <= DESCRIPTION_MAX {
        return text;
    }
    let mut short: String = text.chars().take(DESCRIPTION_MAX - 1).collect();
    short.push('…');
    short
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptions_come_from_the_agents_folder_and_stay_short() {
        let root = tempfile::tempdir().expect("dir");
        let ws = root.path();
        let skill = ws.join(".claude/skills/notes");
        std::fs::create_dir_all(&skill).expect("skill");
        std::fs::write(
            skill.join("SKILL.md"),
            "---\ndescription: Keeps notes.\n---\n",
        )
        .expect("md");
        let commands = ws.join(".claude/plugins/vercel/commands");
        std::fs::create_dir_all(&commands).expect("commands");
        let long = "x".repeat(300);
        std::fs::write(
            commands.join("deploy.md"),
            format!("---\ndescription: {long}\n---\n"),
        )
        .expect("md");
        assert_eq!(describe(ws, "notes"), "Keeps notes.");
        assert!(own_skill(ws, "notes"));
        let deploy = describe(ws, "vercel:deploy");
        assert_eq!(deploy.chars().count(), DESCRIPTION_MAX);
        assert!(deploy.ends_with('…'));
        assert_eq!(describe(ws, "code-review"), "");
        // A name that would leave the folder reads nothing.
        assert_eq!(describe(ws, "../secrets"), "");
        assert_eq!(describe(ws, "a:../b"), "");
    }
}
