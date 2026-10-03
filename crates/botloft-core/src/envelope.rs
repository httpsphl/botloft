//! The text a bot reads when a message arrives (spec 9.3). The first line
//! always says who sent it; bots are told so in their rules.

use crate::protocol::{MessageKind, Task};

/// Who the first line names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sender<'a> {
    Owner,
    Bot {
        handle: &'a str,
    },
    /// A bot the owner deleted after it sent the message (spec 7.6): there
    /// is nobody to reply to.
    DeletedBot,
    /// The daemon, e.g. to say a task expired.
    Botloft,
}

#[derive(Debug, Clone, Copy)]
pub struct Envelope<'a> {
    pub from: Sender<'a>,
    /// Display name of the crew.
    pub crew: &'a str,
    pub kind: MessageKind,
    /// The task a `task`, `result` or `system` message is about.
    pub task: Option<&'a Task>,
    pub body: &'a str,
}

impl Envelope<'_> {
    /// Renders the envelope as of `now_ms`, which sets the relative deadline.
    pub fn render(&self, now_ms: i64) -> String {
        let mut header = format!("[botloft] from {} · crew {}", self.sender(), self.crew);
        if let Some(task) = self.task {
            let detail = match self.kind {
                MessageKind::Task => {
                    format!("task {} · {}", task.id, due(task.deadline_at, now_ms))
                }
                MessageKind::Result => format!("result of task {} · {}", task.id, task.status),
                MessageKind::Note | MessageKind::System | MessageKind::Routine => {
                    format!("task {} · {}", task.id, task.status)
                }
            };
            header.push_str(" · ");
            header.push_str(&detail);
        }
        let mut text = header;
        if let Sender::Bot { handle } = self.from {
            text.push_str(&format!("\nReply with send_message(to: \"{handle}\")."));
            if let (MessageKind::Task, Some(task)) = (self.kind, self.task) {
                text.push_str(&format!(
                    " When the task is done, call complete_task(task_id: \"{}\").",
                    task.id
                ));
            }
        }
        text.push_str("\n\n");
        text.push_str(self.body);
        text
    }

    fn sender(&self) -> String {
        match self.from {
            // No `@`: a bot may well be called "Owner".
            Sender::Owner => "the owner".to_owned(),
            Sender::Bot { handle } => format!("@{handle}"),
            Sender::DeletedBot => "a deleted bot".to_owned(),
            Sender::Botloft => "Botloft".to_owned(),
        }
    }
}

/// What a routine's message says (spec 20.5): which routine, the time it
/// is for (absolute: the bot does not know the current time) and that
/// nobody is watching.
#[derive(Debug, Clone, Copy)]
pub struct RoutineEnvelope<'a> {
    pub name: &'a str,
    pub when: RoutineWhen<'a>,
    pub body: &'a str,
}

/// What made a routine run.
#[derive(Debug, Clone, Copy)]
pub enum RoutineWhen<'a> {
    /// Its time came, or the owner clicked "Run now".
    Scheduled {
        /// Local time in the routine's zone, like `2026-10-01 09:00`.
        at: &'a str,
        timezone: &'a str,
    },
    /// A bot of the crew sent the signal it waits for (spec 20.13).
    Signal {
        name: &'a str,
        /// The sender's handle; `None` once that bot was deleted.
        from: Option<&'a str>,
        /// What the sender wrote with it.
        note: Option<&'a str>,
    },
}

impl RoutineEnvelope<'_> {
    pub fn render(&self) -> String {
        let watch = "Nobody is watching live: do the work, then report it in your reply.";
        match self.when {
            RoutineWhen::Scheduled { at, timezone } => format!(
                "[botloft] routine \"{}\" · scheduled {at} ({timezone})\n{watch}\n\n{}",
                self.name, self.body
            ),
            RoutineWhen::Signal { name, from, note } => {
                let sender = from.map_or_else(|| "a deleted bot".to_owned(), |h| format!("@{h}"));
                let mut text = format!(
                    "[botloft] routine \"{}\" · signal \"{name}\" from {sender}\n{watch}\n\n{}",
                    self.name, self.body
                );
                if let Some(note) = note {
                    // A bot wrote it, not the owner: it informs, it does not instruct.
                    text.push_str(&format!(
                        "\n\nNote {sender} sent with the signal (from a bot, not the owner):\n{note}"
                    ));
                }
                text
            }
        }
    }
}

/// Longest part of a question quoted above the owner's answer, in
/// characters.
const QUOTED_QUESTION_MAX_CHARS: usize = 300;

/// The owner's answer to a bot's question (spec 23.4). No `[botloft]`: it is
/// the owner speaking, after a line that says what it answers.
pub fn answer(question_id: &str, question: &str, body: &str) -> String {
    let quoted = crate::chat::one_line(question, QUOTED_QUESTION_MAX_CHARS);
    format!(
        "Answer to your question {question_id}: \"{quoted}\"

{body}"
    )
}

/// Time left until `deadline_ms`, rounded for a reader who does not know
/// the current time.
pub fn due(deadline_ms: i64, now_ms: i64) -> String {
    let left = deadline_ms.saturating_sub(now_ms);
    if left <= 0 {
        return "overdue".to_owned();
    }
    let minutes = (left + 59_999) / 60_000;
    if minutes < 90 {
        format!("due in {minutes} min")
    } else {
        format!("due in {} h", (minutes + 30) / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{BotId, CrewId, TaskId};
    use crate::protocol::TaskStatus;

    const MINUTE: i64 = 60_000;

    fn task(deadline_at: i64, status: TaskStatus) -> Task {
        Task {
            id: "tsk_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0"
                .parse::<TaskId>()
                .expect("id"),
            crew_id: CrewId::generate(),
            requester_bot_id: BotId::generate(),
            assignee_bot_id: BotId::generate(),
            status,
            deadline_at,
            hops: 1,
            origin_task_id: None,
            result: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    fn envelope<'a>(from: Sender<'a>, kind: MessageKind, task: Option<&'a Task>) -> Envelope<'a> {
        Envelope {
            from,
            crew: "Site",
            kind,
            task,
            body: "Check the build.\nThanks!",
        }
    }

    #[test]
    fn a_task_names_its_id_deadline_and_how_to_finish_it() {
        let task = task(120 * MINUTE, TaskStatus::Open);
        let text = envelope(
            Sender::Bot { handle: "revisor" },
            MessageKind::Task,
            Some(&task),
        )
        .render(0);
        assert_eq!(
            text,
            "[botloft] from @revisor · crew Site · task tsk_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0 · due in 2 h\n\
             Reply with send_message(to: \"revisor\"). When the task is done, \
             call complete_task(task_id: \"tsk_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0\").\n\
             \n\
             Check the build.\nThanks!"
        );
    }

    #[test]
    fn owner_notes_have_no_reply_line() {
        let text = envelope(Sender::Owner, MessageKind::Note, None).render(0);
        assert_eq!(
            text,
            "[botloft] from the owner · crew Site\n\nCheck the build.\nThanks!"
        );
    }

    #[test]
    fn a_deleted_sender_cannot_be_answered() {
        let text = envelope(Sender::DeletedBot, MessageKind::Note, None).render(0);
        assert_eq!(
            text,
            "[botloft] from a deleted bot · crew Site\n\nCheck the build.\nThanks!"
        );
    }

    #[test]
    fn results_and_notices_carry_the_task_status() {
        let done = task(0, TaskStatus::Done);
        let result = envelope(
            Sender::Bot { handle: "writer" },
            MessageKind::Result,
            Some(&done),
        )
        .render(0);
        assert!(result.starts_with(
            "[botloft] from @writer · crew Site · result of task tsk_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0 · done\n\
             Reply with send_message(to: \"writer\").\n\n"
        ));
        let expired = task(0, TaskStatus::Expired);
        let notice = envelope(Sender::Botloft, MessageKind::System, Some(&expired)).render(0);
        assert!(notice.starts_with(
            "[botloft] from Botloft · crew Site · task tsk_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0 · expired\n\n"
        ));
    }

    #[test]
    fn deadlines_read_as_time_left() {
        assert_eq!(due(0, 0), "overdue");
        assert_eq!(due(1, 0), "due in 1 min");
        assert_eq!(due(45 * MINUTE, 0), "due in 45 min");
        assert_eq!(due(89 * MINUTE, 0), "due in 89 min");
        assert_eq!(due(90 * MINUTE, 0), "due in 2 h");
        assert_eq!(due(26 * 60 * MINUTE, 0), "due in 26 h");
        assert_eq!(due(0, 5 * MINUTE), "overdue");
    }

    #[test]
    fn a_routine_says_what_and_when_and_that_nobody_watches() {
        let text = RoutineEnvelope {
            name: "Resumo da manhã",
            when: RoutineWhen::Scheduled {
                at: "2026-10-01 09:00",
                timezone: "America/Sao_Paulo",
            },
            body: "Summarize shared/inbox.",
        }
        .render();
        assert_eq!(
            text,
            "[botloft] routine \"Resumo da manhã\" · scheduled 2026-10-01 09:00 (America/Sao_Paulo)\n\
             Nobody is watching live: do the work, then report it in your reply.\n\n\
             Summarize shared/inbox."
        );
    }

    #[test]
    fn a_signal_says_who_sent_it_and_keeps_the_note_apart() {
        let text = RoutineEnvelope {
            name: "Review",
            when: RoutineWhen::Signal {
                name: "report-ready",
                from: Some("writer"),
                note: Some("It is in shared/report.md"),
            },
            body: "Review the report.",
        }
        .render();
        assert_eq!(
            text,
            "[botloft] routine \"Review\" · signal \"report-ready\" from @writer\n\
             Nobody is watching live: do the work, then report it in your reply.\n\n\
             Review the report.\n\n\
             Note @writer sent with the signal (from a bot, not the owner):\n\
             It is in shared/report.md"
        );
        let gone = RoutineEnvelope {
            name: "Review",
            when: RoutineWhen::Signal {
                name: "report-ready",
                from: None,
                note: None,
            },
            body: "Review the report.",
        }
        .render();
        assert!(gone.starts_with(
            "[botloft] routine \"Review\" · signal \"report-ready\" from a deleted bot\n"
        ));
        assert!(gone.ends_with("Review the report."));
    }
}
