//! `ask_chrome` (spec 37.5): an agent asks the owner for their own Chrome,
//! the Claude in Chrome extension, when a task needs a site that only works
//! there or one where only the owner is signed in. The owner allows or
//! declines in the agent's chat, after reading the risks, and the call
//! waits; allowed, the agent gets the Chrome when it starts again, once its
//! turn ends, and a note tells it to go on.

use botloft_core::chat::CHROME_TOOL;
use botloft_core::ids::BotId;
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::{explain, parse, tool_result};
use crate::approvals::{self, Answer};
use crate::state::Daemon;

pub const ASK_CHROME: &str = "ask_chrome";
const WHY_MAX: usize = 300;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    why: String,
}

pub(super) fn tool() -> Value {
    json!({
        "name": ASK_CHROME,
        "title": "Ask for the owner's own Chrome",
        "description": "Asks the owner to let you use their own Chrome browser, through the \
            Claude in Chrome extension, signed in to their accounts. Use it only when a task \
            needs it: a site that works only in the owner's Chrome, or one where only the \
            owner is signed in. For anything else, use your own browser. The owner allows or \
            declines it in your chat; the call waits for that. Allowed, the Chrome comes when \
            you start again, right after this turn: end the turn, and a note will tell you to \
            go on.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "why": {
                    "type": "string",
                    "maxLength": WHY_MAX,
                    "description": "For the owner: what you need it for and on which site, \
                        in a sentence.",
                },
            },
            "required": ["why"],
            "additionalProperties": false,
        },
    })
}

pub(super) async fn ask(daemon: &Daemon, bot: &BotId, generation: u64, args: Value) -> Value {
    tool_result(run(daemon, bot, generation, args).await)
}

async fn run(daemon: &Daemon, bot: &BotId, generation: u64, args: Value) -> Result<Value, String> {
    let args: Args = parse(args)?;
    let why = args.why.trim();
    if why.is_empty() {
        return Err("why must say what you need the Chrome for".to_owned());
    }
    if why.chars().count() > WHY_MAX {
        return Err(format!("why must be at most {WHY_MAX} characters"));
    }
    if !crate::connectors::on_claude(daemon, bot) {
        return Err("Only agents on Claude Code can use the owner's Chrome.".to_owned());
    }
    if crate::chrome::given(daemon, bot) {
        return Ok(json!({
            "done": true,
            "note": "You already have the owner's Chrome: its tools are mcp__claude-in-chrome__*.",
        }));
    }
    let input = json!({ "why": why });
    match approvals::ask(daemon, bot, generation, CHROME_TOOL, &input, "").await {
        Some(Answer::Allowed { .. }) => {
            crate::chrome::grant(daemon, bot).map_err(explain)?;
            Ok(json!({
                "done": true,
                "note": "The owner gave you their Chrome. It comes when you start again: end \
                    this turn now, saying you will go on in a moment. A note will tell you when \
                    it is ready.",
            }))
        }
        Some(Answer::Denied { note }) => {
            let said = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Ok(json!({
                "done": false,
                "note": format!("The owner declined to give you their Chrome.{said}"),
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
