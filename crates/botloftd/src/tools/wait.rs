//! `wait_for` (spec 20.14): the bot asks Botloft to wake it once, when a
//! file appears, a program ends or a page shows some words, or when the
//! time runs out. Botloft looks by itself, without the model.

use botloft_core::protocol::{RoutinesCreateParams, Schedule};
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::explain;
use super::routine::{PROMPT_MAX, system_zone};
use super::wait_on::{self, TEXT_MAX_CHARS, describe};
use crate::routines::schedule::Plan;
use crate::seat::Seat;
use crate::service::routines;
use crate::state::Daemon;

pub const WAIT_FOR: &str = "wait_for";

const HOURS_MAX: u32 = 168;
const HOUR_MS: i64 = 3_600_000;
/// Waits a bot may have open at once.
const WAITS_PER_BOT: u32 = 5;

pub(super) fn tool() -> Value {
    json!({
        "name": WAIT_FOR,
        "title": "Wait for something",
        "description": "Asks Botloft to wake you once, when a file appears, a program ends or a \
            web page shows some words, or when the time runs out. Botloft looks by itself every \
            few seconds (a page once a minute) without using the owner's plan. Use it instead \
            of sleep in a command or coming back again and again to check, then end your turn. \
            You wake with a message that starts with [botloft] wait and says whether it \
            happened, then what you wrote in `then`. For another bot of your crew, use \
            send_signal and a routine instead.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "maxLength": 80,
                    "description": "A short name, in the owner's language, like \"Wait for the \
                        report\". The owner sees it in your chat.",
                },
                "what": {
                    "type": "string",
                    "enum": ["file", "program", "page"],
                    "description": "file: a file appears (give path). program: a program that \
                        runs now ends (give program). page: a page shows some words (give url \
                        and text).",
                },
                "path": {
                    "type": "string",
                    "description": "For file: absolute, or relative to your folder; it must be \
                        in your folder or the crew's work folder and not exist yet.",
                },
                "program": {
                    "type": "string",
                    "description": "For program: its name, like ffmpeg or ffmpeg.exe, without \
                        a folder.",
                },
                "url": {
                    "type": "string",
                    "description": "For page: an http or https address that opens without a \
                        login.",
                },
                "text": {
                    "type": "string",
                    "maxLength": TEXT_MAX_CHARS,
                    "description": "For page: the words it should show; case and accents do \
                        not matter.",
                },
                "hours": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": HOURS_MAX,
                    "description": "How long to wait at most, in hours, up to 168 (7 days).",
                },
                "then": {
                    "type": "string",
                    "maxLength": PROMPT_MAX,
                    "description": "What you will do when you wake, written for yourself.",
                },
            },
            "required": ["name", "what", "hours", "then"],
            "additionalProperties": false,
        },
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Args {
    name: String,
    what: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    program: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    text: Option<String>,
    hours: u32,
    then: String,
}

pub(super) fn call(daemon: &Daemon, bot: &Seat, args: Args) -> Result<Value, String> {
    if !(1..=HOURS_MAX).contains(&args.hours) {
        return Err(format!("hours must be between 1 and {HOURS_MAX}"));
    }
    if args.then.chars().count() > PROMPT_MAX {
        return Err(format!("then must be at most {PROMPT_MAX} characters"));
    }
    let on = match (
        args.what.as_str(),
        &args.path,
        &args.program,
        &args.url,
        &args.text,
    ) {
        ("file", Some(path), None, None, None) => wait_on::file(daemon, bot, path)?,
        ("program", None, Some(name), None, None) => wait_on::program(daemon, name)?,
        ("page", None, None, Some(url), Some(text)) => wait_on::page(url, text)?,
        ("file", ..) => return Err("what: file takes path, and nothing else".to_owned()),
        ("program", ..) => return Err("what: program takes program, and nothing else".to_owned()),
        ("page", ..) => return Err("what: page takes url and text, and nothing else".to_owned()),
        _ => return Err("what must be file, program or page".to_owned()),
    };
    let open = daemon
        .store()
        .open_waits(bot)
        .map_err(|err| explain(err.into()))?;
    if open >= WAITS_PER_BOT {
        return Err(format!(
            "you already have {WAITS_PER_BOT} waits open; let one end first, or ask the owner to \
             cancel one"
        ));
    }

    let until = daemon.clock.now_ms() + i64::from(args.hours) * HOUR_MS;
    let waiting_for = describe(&on);
    let schedule = Schedule::Wait { on, until };
    let timezone = system_zone();
    let until_text = Plan::new(&schedule, &timezone)
        .map(|plan| plan.local_time(until))
        .unwrap_or_default();
    let routine = routines::create_wait(
        daemon,
        RoutinesCreateParams {
            bot_id: bot.bot.clone(),
            name: args.name,
            prompt: args.then,
            schedule,
            timezone: timezone.clone(),
            overlap: None,
            missed: None,
        },
    )
    .map_err(explain)?;
    crate::service::replies::keep_routine(daemon, bot, &routine);
    Ok(json!({
        "created": true,
        "routine_id": routine.id,
        "waiting_for": waiting_for,
        "until": until_text,
        "timezone": timezone,
        "note": "Botloft looks by itself, without using the owner's plan. You wake once, with a \
            message that starts with [botloft] wait: when it happens, or when the time runs out. \
            End your turn now and do not check it yourself. The owner sees the wait in your \
            chat and may cancel it.",
    }))
}
