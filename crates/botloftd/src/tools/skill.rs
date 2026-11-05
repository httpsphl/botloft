//! `ask_skill` (spec 38.5): an agent asks the owner for one of the skills
//! of the owner's own Claude Code, listed in its rules. The owner allows or
//! declines in the agent's chat and the call waits; allowed, the agent gets
//! the skill when it starts again, once its turn ends, and a note tells it
//! to go on.

use botloft_core::chat::SKILL_TOOL;
use botloft_core::ids::BotId;
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::{explain, parse, tool_result};
use crate::approvals::{self, Answer};
use crate::state::Daemon;

pub const ASK_SKILL: &str = "ask_skill";
const WHY_MAX: usize = 300;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    name: String,
    why: Option<String>,
}

pub(super) fn tool() -> Value {
    json!({
        "name": ASK_SKILL,
        "title": "Ask for one of the owner's skills",
        "description": "Asks the owner to give you one of the skills of their own Claude Code, \
            listed in your rules. The owner allows or declines it in your chat; the call waits \
            for that. Allowed, the skill comes when you start again, right after this turn: \
            end the turn, and a note will tell you to go on.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The skill's name as your rules list it.",
                },
                "why": {
                    "type": "string",
                    "maxLength": WHY_MAX,
                    "description": "For the owner: what you need it for, in a sentence.",
                },
            },
            "required": ["name"],
            "additionalProperties": false,
        },
    })
}

pub(super) async fn ask(daemon: &Daemon, bot: &BotId, generation: u64, args: Value) -> Value {
    tool_result(run(daemon, bot, generation, args).await)
}

async fn run(daemon: &Daemon, bot: &BotId, generation: u64, args: Value) -> Result<Value, String> {
    let args: Args = parse(args)?;
    if args
        .why
        .as_ref()
        .is_some_and(|why| why.chars().count() > WHY_MAX)
    {
        return Err(format!("why must be at most {WHY_MAX} characters"));
    }
    if !crate::connectors::on_claude(daemon, bot) {
        return Err("Only agents on Claude Code can use the owner's skills.".to_owned());
    }
    let name = crate::skills::known(daemon, &args.name).ok_or_else(|| {
        format!(
            "\"{}\" is not one of the owner's skills. Ask for one your rules list.",
            args.name.trim()
        )
    })?;
    if crate::skills::given(daemon, bot, &name) {
        return Ok(json!({
            "done": true,
            "note": format!("You already have {name}: it is in .claude/skills."),
        }));
    }
    let input = json!({ "name": name, "why": args.why });
    match approvals::ask(daemon, bot, generation, SKILL_TOOL, &input, "").await {
        Some(Answer::Allowed { .. }) => {
            crate::skills::grant(daemon, bot, &name).map_err(explain)?;
            Ok(json!({
                "done": true,
                "note": format!(
                    "The owner gave you {name}. It comes when you start again: end this turn \
                     now, saying you will go on in a moment. A note will tell you when it is \
                     ready."
                ),
            }))
        }
        Some(Answer::Denied { note }) => {
            let said = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Ok(json!({
                "done": false,
                "note": format!("The owner declined {name}.{said}"),
            }))
        }
        Some(Answer::Expired) => Ok(json!({
            "done": false,
            "note": "The owner did not answer in time. Tell the owner, and ask again later if it \
                still helps.",
        })),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}
