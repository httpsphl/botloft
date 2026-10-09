//! `crew_activity` (spec 29.2): what the bots of a crew did lately, for the
//! chief to write the owner a summary. Counts and short excerpts of the
//! tasks the bots gave each other; never the chat with the owner.

use botloft_core::ids::BotId;
use botloft_core::protocol::{Bot, BotsListParams, Task, TaskStatus};
use botloft_store::TaskFilter;
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::explain;
use crate::service::{bots, lead};
use crate::state::Daemon;

pub const CREW_ACTIVITY: &str = "crew_activity";

const HOURS_DEFAULT: u32 = 24;
const HOURS_MAX: u32 = 168;
/// Longest excerpt of a request or a result, in characters.
const EXCERPT_CHARS: usize = 400;
/// Tasks listed for each bot; the counts always cover all of them.
const TASKS_PER_BOT: usize = 6;
const ROUTINES_PER_BOT: usize = 5;
const HOUR_MS: i64 = 3_600_000;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Args {
    #[serde(default)]
    hours: Option<u32>,
}

pub(super) fn tools() -> [Value; 1] {
    [json!({
        "name": CREW_ACTIVITY,
        "title": "Crew activity",
        "description": "Only for the crew's chief. What each bot of the crew did in the last \
            hours (24 by default, up to 168): tasks finished, failed or expired, with what was \
            asked and the result in short; tasks still open and the ones past their deadline; \
            what is waiting for the owner (approvals and questions) and the latest run of each \
            routine. Use it to write the owner a summary. Texts are shortened, and the chats \
            with the owner are not included.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "hours": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": HOURS_MAX,
                    "description": "How far back to look, in hours. 24 if left out.",
                },
            },
            "additionalProperties": false,
        },
        "annotations": { "readOnlyHint": true },
    })]
}

pub(super) fn call(daemon: &Daemon, bot: &BotId, args: Args) -> Result<Value, String> {
    let hours = args.hours.unwrap_or(HOURS_DEFAULT);
    if !(1..=HOURS_MAX).contains(&hours) {
        return Err(format!("hours must be between 1 and {HOURS_MAX}"));
    }
    let (crew, me) = bots::active(&daemon.store(), bot).map_err(explain)?;
    if !lead::is_lead(&crew, &me.id) {
        return Err(
            "only the crew's chief can read the crew's activity; ask the chief with send_message"
                .to_owned(),
        );
    }
    let crew_bots = bots::list(
        daemon,
        BotsListParams {
            crew_id: Some(crew.id.clone()),
        },
    )
    .map_err(explain)?;
    let now = daemon.clock.now_ms();
    let since = now - i64::from(hours) * HOUR_MS;

    let store = daemon.store();
    let tasks = store
        .tasks(TaskFilter {
            crew: Some(&crew.id),
            ..TaskFilter::default()
        })
        .map_err(|err| explain(err.into()))?;
    let handle_of = |id: &BotId| {
        crew_bots
            .iter()
            .find(|other| &other.id == id)
            .map_or_else(|| "another crew".to_owned(), |other| other.handle.clone())
    };

    let mut totals = Totals::default();
    let mut report = Vec::new();
    for member in &crew_bots {
        let mine: Vec<&Task> = tasks
            .iter()
            .filter(|task| task.assignee_bot_id == member.id)
            .collect();
        let finished: Vec<&Task> = mine
            .iter()
            .copied()
            .filter(|task| task.status != TaskStatus::Open && task.updated_at >= since)
            .collect();
        let count =
            |status: TaskStatus| finished.iter().filter(|task| task.status == status).count();
        let open = mine
            .iter()
            .filter(|task| task.status == TaskStatus::Open)
            .count();
        let overdue = mine
            .iter()
            .filter(|task| task.status == TaskStatus::Open && task.deadline_at < now)
            .count();
        let waiting_approvals = store
            .pending_approvals(&member.id)
            .map_or(0, |pending| pending.len());
        let waiting_questions = store.open_questions(&member.id).unwrap_or(0);
        totals.add(
            count(TaskStatus::Done),
            count(TaskStatus::Failed),
            count(TaskStatus::Expired),
            open,
            overdue,
            waiting_approvals + waiting_questions,
        );

        let recent: Vec<Value> = finished
            .iter()
            .take(TASKS_PER_BOT)
            .map(|task| {
                let request = store
                    .task_request(&task.id)
                    .ok()
                    .flatten()
                    .map(|message| excerpt(&message.body));
                json!({
                    "status": task.status,
                    "from": handle_of(&task.requester_bot_id),
                    "hours_ago": hours_ago(now, task.updated_at),
                    "request": request,
                    "result": task.result.as_deref().map(excerpt),
                })
            })
            .collect();
        let routines: Vec<Value> = store
            .routines(Some(&member.id), false)
            .unwrap_or_default()
            .iter()
            .take(ROUTINES_PER_BOT)
            .map(|routine| {
                json!({
                    "name": routine.name,
                    "on": routine.enabled,
                    "last_run": routine.last_run.as_ref().map(|run| json!({
                        "status": run.status,
                        "hours_ago": hours_ago(now, run.created_at),
                    })),
                })
            })
            .collect();

        report.push(bot_report(
            member,
            json!({
                "done": count(TaskStatus::Done),
                "failed": count(TaskStatus::Failed),
                "expired": count(TaskStatus::Expired),
                "open": open,
                "overdue": overdue,
            }),
            recent,
            json!({ "approvals": waiting_approvals, "questions": waiting_questions }),
            routines,
        ));
    }

    Ok(json!({
        "crew": crew.name,
        "window_hours": hours,
        "totals": totals.json(),
        "bots": report,
        "note": "Counts cover everything in the window; `recent` lists only the latest few \
            tasks of each bot. Write the owner a short summary in their language and say \
            plainly what you could not see.",
    }))
}

fn bot_report(
    member: &Bot,
    tasks: Value,
    recent: Vec<Value>,
    waiting_for_owner: Value,
    routines: Vec<Value>,
) -> Value {
    json!({
        "handle": member.handle,
        "name": member.name,
        "role": member.role,
        "state": member.state,
        "tasks": tasks,
        "recent": recent,
        "waiting_for_owner": waiting_for_owner,
        "routines": routines,
    })
}

#[derive(Default)]
struct Totals {
    done: usize,
    failed: usize,
    expired: usize,
    open: usize,
    overdue: usize,
    waiting_for_owner: usize,
}

impl Totals {
    fn add(
        &mut self,
        done: usize,
        failed: usize,
        expired: usize,
        open: usize,
        overdue: usize,
        waiting: usize,
    ) {
        self.done += done;
        self.failed += failed;
        self.expired += expired;
        self.open += open;
        self.overdue += overdue;
        self.waiting_for_owner += waiting;
    }

    fn json(&self) -> Value {
        json!({
            "done": self.done,
            "failed": self.failed,
            "expired": self.expired,
            "open": self.open,
            "overdue": self.overdue,
            "waiting_for_owner": self.waiting_for_owner,
        })
    }
}

/// The text cut to a length a summary can carry.
fn excerpt(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= EXCERPT_CHARS {
        return trimmed.to_owned();
    }
    let cut: String = trimmed.chars().take(EXCERPT_CHARS).collect();
    format!("{}…", cut.trim_end())
}

/// Whole and tenth hours between `then` and `now`, never below zero.
fn hours_ago(now: i64, then: i64) -> f64 {
    let tenths = ((now - then).max(0) * 10 + HOUR_MS / 2) / HOUR_MS;
    tenths as f64 / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_text_is_cut_and_marked() {
        let long = "a".repeat(EXCERPT_CHARS + 50);
        let cut = excerpt(&long);
        assert_eq!(cut.chars().count(), EXCERPT_CHARS + 1);
        assert!(cut.ends_with('…'));
        assert_eq!(excerpt("  short  "), "short");
        // Not in the middle of a multi-byte character.
        assert!(excerpt(&"ã".repeat(EXCERPT_CHARS + 5)).ends_with('…'));
    }

    #[test]
    fn hours_ago_rounds_to_a_tenth() {
        assert_eq!(hours_ago(HOUR_MS * 3, 0), 3.0);
        assert_eq!(hours_ago(HOUR_MS * 3 / 2, 0), 1.5);
        assert_eq!(hours_ago(0, HOUR_MS), 0.0);
    }
}
