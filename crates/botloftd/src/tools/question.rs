//! `ask_owner` (spec 23.2): the bot asks the owner something and carries
//! on. The answer comes back later as the owner's message. `tell_owner`
//! (23.10): the bot leaves the owner a note, and asks nothing.

use botloft_core::protocol::{
    NOTES_PER_HOUR, QUESTION_MAX_CHARS, QUESTION_OPTION_MAX_CHARS, QUESTION_OPTIONS_MAX,
    QUESTION_OPTIONS_MIN, QuestionKind,
};
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::explain;
use crate::seat::Seat;
use crate::service::questions::{self, Ask};
use crate::state::Daemon;

pub const ASK_OWNER: &str = "ask_owner";
pub const TELL_OWNER: &str = "tell_owner";

pub(super) fn tool() -> Value {
    json!({
        "name": ASK_OWNER,
        "title": "Ask the owner",
        "description": "Asks the owner a question when you need their decision or information to \
            go on. It does not wait: the question goes to their question box and your chat, and \
            their answer arrives later as a new message. Use it instead of only writing the \
            question in your reply, above all in a routine or a task from another bot, when \
            nobody is reading your chat. Then carry on with whatever does not depend on the \
            answer, or end your turn; never guess the answer. Never ask for passwords, codes or \
            card numbers.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "question": {
                    "type": "string",
                    "maxLength": QUESTION_MAX_CHARS,
                    "description": "One question, short and clear on its own, in the language \
                        the owner writes to you in. Markdown is fine.",
                },
                "options": {
                    "type": "array",
                    "items": { "type": "string", "maxLength": QUESTION_OPTION_MAX_CHARS },
                    "minItems": QUESTION_OPTIONS_MIN,
                    "maxItems": QUESTION_OPTIONS_MAX,
                    "description": "Ready answers the owner can pick with one click, when the \
                        possible answers are few. They can still write something else.",
                },
            },
            "required": ["question"],
            "additionalProperties": false,
        },
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AskArgs {
    question: String,
    #[serde(default)]
    options: Vec<String>,
}

pub(super) fn ask(daemon: &Daemon, bot: &Seat, args: AskArgs) -> Result<Value, String> {
    let ask = Ask {
        kind: QuestionKind::Question,
        text: args.question,
        options: args.options,
    };
    let question = questions::ask(daemon, bot, ask).map_err(explain)?;
    Ok(json!({
        "question_id": question.id,
        "note": format!(
            "The owner sees your question in your chat and in their question box. Their answer \
             arrives later as a new message that starts with \"Answer to your question {}\". \
             Don't wait for it and don't guess it: carry on with what doesn't depend on it, or \
             end your turn.",
            question.id
        ),
    }))
}

pub(super) fn note_tool() -> Value {
    json!({
        "name": TELL_OWNER,
        "title": "Tell the owner",
        "description": format!(
            "Leaves the owner a short note they should see soon, when they may not be reading \
             your chat: a routine found what they asked to hear about, long work is done, or \
             something went wrong that they must fix. It goes to their inbox and your chat, with \
             a notice on their computer and phone. It asks nothing: if you need an answer, use \
             ask_owner. Not for every step, and not for what you just answered them in this \
             chat. At most {NOTES_PER_HOUR} an hour."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "note": {
                    "type": "string",
                    "maxLength": QUESTION_MAX_CHARS,
                    "description": "What the owner should know, short and clear on its own, in \
                        the language the owner writes to you in. Markdown is fine; say where \
                        anything you made is.",
                },
            },
            "required": ["note"],
            "additionalProperties": false,
        },
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TellArgs {
    note: String,
}

pub(super) fn tell(daemon: &Daemon, bot: &Seat, args: TellArgs) -> Result<Value, String> {
    let ask = Ask {
        kind: QuestionKind::Note,
        text: args.note,
        options: Vec::new(),
    };
    let note = questions::ask(daemon, bot, ask).map_err(explain)?;
    Ok(json!({
        "note_id": note.id,
        "note": format!(
            "The owner gets your note in their inbox and in your chat. Don't wait for a reply: \
             if they reply to it, that arrives later as a new message that starts with \"Reply \
             to your note {}\".",
            note.id
        ),
    }))
}
