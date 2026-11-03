//! `ask_connector` (spec 34.6): an agent asks the owner for one of the tools
//! of the owner's claude.ai account, instead of signing in to the service in
//! its browser. The owner allows or declines in the agent's chat and the
//! call waits; allowed, the agent gets the tool when it starts again, once
//! its turn ends, and a note tells it to go on.

use botloft_core::chat::CONNECTOR_TOOL;
use botloft_core::ids::BotId;
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::{explain, parse, tool_result};
use crate::approvals::{self, Answer};
use crate::connectors;
use crate::state::Daemon;

pub const ASK_CONNECTOR: &str = "ask_connector";
const WHY_MAX: usize = 300;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    name: String,
    why: Option<String>,
}

pub(super) fn tool() -> Value {
    json!({
        "name": ASK_CONNECTOR,
        "title": "Ask for a tool of the owner's claude.ai account",
        "description": "Asks the owner to let you use one of the tools of their claude.ai \
            account, such as Google Calendar, Gmail or Google Drive, listed in your rules. \
            Use it instead of opening the service in your browser and asking the owner to sign \
            in. The owner allows or declines it in your chat; the call waits for that. Allowed, \
            the tool comes when you start again, right after this turn: end the turn, and a \
            note will tell you to go on.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The tool's name as your rules list it: \"Google Calendar\".",
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
    if !connectors::on_claude(daemon, bot) {
        return Err(
            "Only agents on Claude Code can use the tools of the owner's claude.ai account."
                .to_owned(),
        );
    }
    let name = connectors::account_tool(daemon, &args.name).ok_or_else(|| {
        format!(
            "\"{}\" is not a tool of the owner's claude.ai account. Tell the owner they can \
             connect it on claude.ai, or in Botloft under Settings > Tools > Discover, then \
             ask again. Do not sign in to it in your browser.",
            args.name.trim()
        )
    })?;
    if connectors::given(daemon, bot, &name) {
        return Ok(json!({
            "done": true,
            "note": format!("You already have {name}: its tools are mcp__{}__*.",
                connectors::names::connector_server(&name)),
        }));
    }
    let input = json!({ "name": name, "why": args.why });
    match approvals::ask(daemon, bot, generation, CONNECTOR_TOOL, &input, "").await {
        Some(Answer::Allowed { .. }) => {
            connectors::grant(daemon, bot, &name).map_err(explain)?;
            Ok(json!({
                "done": true,
                "note": format!(
                    "The owner gave you {name}. Its tools come when you start again: end this \
                     turn now, saying you will go on in a moment. A note will tell you when the \
                     tools are ready."
                ),
            }))
        }
        Some(Answer::Denied { note }) => {
            let why = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Ok(json!({
                "done": false,
                "note": format!("The owner declined {name}.{why} Do not use the browser for it instead."),
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
