//! `suggest_bot` (spec 10.2): the crew's chief asks for a new bot. The owner
//! sees the suggestion in the chief's chat, may change it, and allows or
//! declines it; the call waits for that, like a permission request. A chief
//! that bypasses permissions creates the bot at once.

use botloft_core::chat::SUGGEST_TOOL;
use botloft_core::ids::BotId;
use botloft_core::protocol::{
    AgentKind, Bot, BotEffort, BotModel, BotsSetEffortParams, PermissionMode,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::calls::{explain, parse, tool_result};
use crate::approvals::{self, Answer};
use crate::catalog;
use crate::service::bots::{self, NewBot};
use crate::service::{lead, models};
use crate::state::Daemon;

/// Longest instructions a chief may write for a new bot, in characters.
pub(super) const INSTRUCTIONS_MAX: usize = 8_000;
/// Longest reason, in characters.
pub(super) const REASON_MAX: usize = 1_000;

/// The tool's input, and what the owner sends back when they change it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Suggestion {
    name: String,
    #[serde(default)]
    role: String,
    #[serde(default)]
    instructions: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    effort: Option<String>,
    reason: String,
    /// A role of the bot catalog (spec 26.4); gone once it is resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    template: Option<String>,
}

pub(super) async fn suggest(
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
    let mut suggestion: Suggestion = parse(arguments)?;
    resolve(&mut suggestion)?;
    let new = check(&suggestion, bots::default_agent(daemon))?;
    // Nothing impossible goes to the owner.
    lead::check_suggestion(daemon, bot, &new).map_err(explain)?;
    let mode = bots::find(&daemon.store(), bot)
        .map_err(explain)?
        .permission_mode;
    if mode == PermissionMode::BypassPermissions {
        let created = create(daemon, bot, new, &suggestion)?;
        return Ok(created_note(
            &created,
            "You bypass permissions, so it was created without asking the owner.",
        ));
    }

    let input = serde_json::to_value(&suggestion).map_err(|err| err.to_string())?;
    match approvals::ask(daemon, bot, generation, SUGGEST_TOOL, &input, "").await {
        Some(Answer::Allowed { input: None }) => {
            let created = create(daemon, bot, new, &suggestion)?;
            Ok(created_note(&created, "The owner approved it."))
        }
        Some(Answer::Allowed {
            input: Some(changed),
        }) => {
            let changed: Suggestion = serde_json::from_str(&changed)
                .map_err(|err| format!("the owner's changes could not be read: {err}"))?;
            let created = create(
                daemon,
                bot,
                check(&changed, bots::default_agent(daemon))?,
                &changed,
            )?;
            Ok(created_note(
                &created,
                "The owner changed the suggestion before approving it: read the name, role, \
                 model and effort above.",
            ))
        }
        Some(Answer::Denied { note }) => {
            let why = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Ok(json!({
                "created": false,
                "note": format!(
                    "The owner declined this bot.{why} Do the work yourself or with the bots \
                     you have."
                ),
            }))
        }
        Some(Answer::Expired) => Ok(json!({
            "created": false,
            "note": "The owner did not answer in time. Do the work yourself or with the bots you \
                have, and suggest it again later if it still helps.",
        })),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}

/// Fills a suggestion that names a catalog role (spec 26.4): the sheet's role,
/// instructions, model and effort stand in for what is missing, and what the
/// chief wrote goes after the sheet's instructions. Afterwards it is a plain
/// suggestion, so the owner's card, and any change the owner makes to it,
/// read the same as for a bot written from scratch.
fn resolve(suggestion: &mut Suggestion) -> Result<(), String> {
    let Some(id) = suggestion.template.take() else {
        if suggestion.role.trim().is_empty() || suggestion.instructions.trim().is_empty() {
            return Err("role and instructions are required unless you give a template".to_owned());
        }
        return Ok(());
    };
    let sheet = catalog::get(id.trim())
        .ok_or_else(|| format!("no bot template named {id}; list them with list_bot_templates"))?;
    if suggestion.role.trim().is_empty() {
        suggestion.role = sheet.role;
    }
    let extra = suggestion.instructions.trim();
    suggestion.instructions = if extra.is_empty() {
        sheet.instructions
    } else {
        format!("{}\n\nFor this crew:\n{extra}", sheet.instructions)
    };
    suggestion
        .model
        .get_or_insert_with(|| sheet.model.as_str().to_owned());
    suggestion
        .effort
        .get_or_insert_with(|| sheet.effort.as_str().to_owned());
    Ok(())
}

/// Creates the bot, then sets the effort it was suggested with.
fn create(
    daemon: &Daemon,
    lead_bot: &BotId,
    new: NewBot,
    suggestion: &Suggestion,
) -> Result<Bot, String> {
    let effort = effort_of(suggestion)?;
    let created = lead::create_suggested(daemon, lead_bot, new).map_err(explain)?;
    if effort == BotEffort::Default {
        return Ok(created);
    }
    models::set_effort(
        daemon,
        BotsSetEffortParams {
            bot_id: created.id,
            effort,
        },
    )
    .map_err(explain)
}

fn effort_of(suggestion: &Suggestion) -> Result<BotEffort, String> {
    match suggestion.effort.as_deref() {
        None => Ok(BotEffort::Default),
        Some(effort) => effort.parse().map_err(|_| {
            "effort must be \"default\", \"low\", \"medium\", \"high\", \"xhigh\" or \"max\""
                .to_owned()
        }),
    }
}

/// The suggestion as a new bot, with the limits of this tool on top of a
/// bot's own.
fn check(suggestion: &Suggestion, agent: AgentKind) -> Result<NewBot, String> {
    effort_of(suggestion)?;
    if suggestion.instructions.chars().count() > INSTRUCTIONS_MAX {
        return Err(format!(
            "instructions must be at most {INSTRUCTIONS_MAX} characters"
        ));
    }
    let reason = suggestion.reason.trim();
    if reason.is_empty() || reason.chars().count() > REASON_MAX {
        return Err(format!(
            "reason must say, in at most {REASON_MAX} characters, why the crew needs this bot"
        ));
    }
    let model = match suggestion.model.as_deref() {
        None => None,
        Some(model) => Some(model.parse::<BotModel>().map_err(|_| {
            "model must be \"default\", \"fable\", \"opus\", \"sonnet\" or \"haiku\"".to_owned()
        })?),
    };
    NewBot::check(
        &suggestion.name,
        &suggestion.role,
        &suggestion.instructions,
        None,
        model,
        Some(agent),
    )
    .map_err(explain)
}

fn created_note(bot: &Bot, how: &str) -> Value {
    json!({
        "created": true,
        "handle": bot.handle,
        "name": bot.name,
        "role": bot.role,
        "model": bot.model,
        "effort": bot.effort,
        "note": format!(
            "{how} @{} is starting now. Give it work with send_message(to: \"{}\", kind: \"task\").",
            bot.handle, bot.handle
        ),
    })
}
