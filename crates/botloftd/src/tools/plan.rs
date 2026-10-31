//! `show_plan` (spec 10.5): before a long task, the bot shows the owner what
//! it understood and how it will do it. The owner has a few seconds to start
//! it now, pick the smaller way, change it or cancel it; with no answer the
//! bot starts by itself.

use std::time::Duration;

use botloft_core::chat::SHOW_PLAN_TOOL;
use botloft_core::ids::BotId;
use botloft_core::protocol::PermissionMode;
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::{explain, parse, tool_result};
use crate::approvals::{self, Answer};
use crate::service::{ApiError, ApiResult, bots};
use crate::state::Daemon;

pub const SHOW_PLAN: &str = "show_plan";
/// How long the owner has before the bot starts by itself.
pub const PLAN_WAIT: Duration = Duration::from_secs(20);
const SUMMARY_MAX: usize = 300;
const STEP_MAX: usize = 200;
const STEPS_MIN: usize = 2;
const STEPS_MAX: usize = 8;
const ALTERNATIVE: &str = "alternative";

pub(super) fn tool() -> Value {
    json!({
        "name": SHOW_PLAN,
        "title": "Show your plan",
        "description": "Shows the owner your plan before long work, when your rules ask for \
            it: what you understood, the steps and, if there is one, a smaller way. The owner \
            has a few seconds to change it; the call waits that long and, with no answer, \
            tells you to start. Do what the result says.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "summary": {
                    "type": "string",
                    "maxLength": SUMMARY_MAX,
                    "description": "One line: what you understood the owner wants, in their \
                        language.",
                },
                "steps": {
                    "type": "array",
                    "items": { "type": "string", "maxLength": STEP_MAX },
                    "minItems": STEPS_MIN,
                    "maxItems": STEPS_MAX,
                    "description": "What you will do, in order, in plain words.",
                },
                "alternative": {
                    "type": "string",
                    "maxLength": SUMMARY_MAX,
                    "description": "A smaller way to do it, if there is one, such as only \
                        listing the problems without changing anything.",
                },
            },
            "required": ["summary", "steps"],
            "additionalProperties": false,
        },
        "annotations": { "readOnlyHint": true },
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    summary: String,
    steps: Vec<String>,
    alternative: Option<String>,
}

/// The fields trimmed and checked; nothing invalid reaches the owner.
fn checked(args: Args) -> Result<Args, String> {
    let line = |name: &str, text: &str, max: usize| -> Result<String, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err(format!("{name} cannot be empty"));
        }
        if text.chars().count() > max {
            return Err(format!("{name} must be at most {max} characters"));
        }
        Ok(text.to_owned())
    };
    if !(STEPS_MIN..=STEPS_MAX).contains(&args.steps.len()) {
        return Err(format!("steps takes {STEPS_MIN} to {STEPS_MAX} steps"));
    }
    Ok(Args {
        summary: line("summary", &args.summary, SUMMARY_MAX)?,
        steps: args
            .steps
            .iter()
            .map(|step| line("a step", step, STEP_MAX))
            .collect::<Result<_, _>>()?,
        alternative: args
            .alternative
            .filter(|text| !text.trim().is_empty())
            .map(|text| line("alternative", &text, SUMMARY_MAX))
            .transpose()?,
    })
}

pub(super) async fn show(daemon: &Daemon, bot: &BotId, generation: u64, args: Value) -> Value {
    tool_result(run(daemon, bot, generation, args).await)
}

async fn run(daemon: &Daemon, bot: &BotId, generation: u64, args: Value) -> Result<Value, String> {
    let args = checked(parse(args)?)?;
    let record = bots::find(&daemon.store(), bot).map_err(explain)?;
    if !record.plan_first {
        return Ok(go("The owner did not ask to see plans first: go ahead."));
    }
    if record.permission_mode == PermissionMode::Plan {
        return Ok(go(
            "You are in plan mode: ExitPlanMode shows the owner your plan. Go on planning.",
        ));
    }
    // Tests make every wait for the owner short; the plan's too.
    let within = PLAN_WAIT.min(daemon.settings.approval_wait());
    let starts_at = daemon.clock.now_ms() + i64::try_from(within.as_millis()).unwrap_or(0);
    let input = json!({
        "summary": args.summary,
        "steps": args.steps,
        "alternative": args.alternative,
        "starts_at": starts_at,
    });
    let answer = approvals::ask_briefly(daemon, bot, generation, SHOW_PLAN_TOOL, &input, within);
    Ok(match answer.await {
        Some(Answer::Allowed { input: Some(_), .. }) => json!({
            "start": ALTERNATIVE,
            "note": "The owner picked your alternative: do only that, not the whole plan.",
        }),
        Some(Answer::Allowed { input: None, .. }) => go("The owner said start: go ahead."),
        Some(Answer::Denied { note: Some(note) }) => json!({
            "start": false,
            "change": note,
            "note": "The owner wants something different: do not start the plan. Do what they \
                said; if the work is still long, show the new plan first.",
        }),
        Some(Answer::Denied { note: None }) => json!({
            "start": false,
            "note": "The owner cancelled: do not start. Ask in one sentence what they want, then \
                end your turn.",
        }),
        Some(Answer::Expired) => go("The owner did not change the plan: go ahead."),
        None => go("Botloft could not show the plan: go ahead."),
    })
}

fn go(note: &str) -> Value {
    json!({ "start": "plan", "note": note })
}

/// The owner's answer when they pick the alternative: `{"choice":
/// "alternative"}`, and only for a plan that has one. It is kept with the
/// plan, so the answered card still shows it.
pub(crate) fn check_answer(saved: &str, answer: &str) -> ApiResult<String> {
    let wrong = || ApiError::validation("the answer to a plan is {\"choice\":\"alternative\"}");
    let answer: Value = serde_json::from_str(answer).map_err(|_| wrong())?;
    if answer != json!({ "choice": ALTERNATIVE }) {
        return Err(wrong());
    }
    let mut plan: Value = serde_json::from_str(saved).unwrap_or_default();
    if !plan["alternative"].is_string() {
        return Err(ApiError::validation("this plan has no alternative"));
    }
    plan["choice"] = json!(ALTERNATIVE);
    Ok(plan.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(steps: &[&str]) -> Args {
        Args {
            summary: "  Fix the login  ".into(),
            steps: steps.iter().map(|step| (*step).to_owned()).collect(),
            alternative: Some(" ".into()),
        }
    }

    #[test]
    fn plans_are_trimmed_and_an_empty_alternative_is_none() {
        let plan = checked(args(&["Read the form", " Fix the check "])).unwrap();
        assert_eq!(plan.summary, "Fix the login");
        assert_eq!(plan.steps, ["Read the form", "Fix the check"]);
        assert_eq!(plan.alternative, None);
    }

    #[test]
    fn plans_need_two_to_eight_short_steps() {
        assert!(checked(args(&["Only one"])).is_err());
        assert!(checked(args(&["a"; 9])).is_err());
        assert!(checked(args(&["a", ""])).is_err());
        let long = "x".repeat(STEP_MAX + 1);
        assert!(checked(args(&["a", &long])).is_err());
    }

    #[test]
    fn only_a_plan_with_an_alternative_takes_the_alternative() {
        let with = r#"{"summary":"s","steps":["a","b"],"alternative":"smaller"}"#;
        let without = r#"{"summary":"s","steps":["a","b"],"alternative":null}"#;
        let pick = r#"{"choice":"alternative"}"#;
        let kept: Value = serde_json::from_str(&check_answer(with, pick).unwrap()).unwrap();
        assert_eq!(kept["choice"], ALTERNATIVE);
        assert_eq!(kept["steps"][1], "b");
        assert!(check_answer(without, pick).is_err());
        assert!(check_answer(with, r#"{"choice":"other"}"#).is_err());
    }
}
