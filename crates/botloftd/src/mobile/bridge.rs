//! Between the phone and the owner's own flows (spec 28.5): what happens in
//! the chat and the question box reaches the phone as a card, and what the
//! phone answers goes through the same code as the app's answer, with a few
//! more rules.

use std::sync::Arc;

use botloft_core::ids::{ApprovalId, QuestionId};
use botloft_core::protocol::{
    ApprovalStatus, ApprovalsAnswerParams, ChatBody, FromPhone, QuestionIdParams, QuestionStatus,
    QuestionsAnswerParams, SEALED_MAX, ToPhone,
};
use serde_json::json;
use tracing::{debug, warn};

use super::cards;
use super::seal::{self, Dir};
use super::{Mobile, identity};
use crate::approvals;
use crate::service::questions;
use crate::state::{Daemon, Event};

/// How much of a snapshot's cards go in one message: the rest follow as
/// cards of their own.
const SNAPSHOT_ROOM: usize = SEALED_MAX - 1024;

impl Mobile {
    /// Something happened in the owner's chat or question box: the phones
    /// that are on hear of it. Phones that are not ask when they open.
    pub(crate) fn on_event(&self, daemon: &Daemon, event: &Event) {
        let message = match event {
            Event::ChatItem(change) => match &change.item.body {
                ChatBody::Approval(shown) if shown.status == ApprovalStatus::Pending => {
                    cards::approval_card(daemon, &change.item, shown)
                        .map(|card| ToPhone::ApprovalOpen { card })
                }
                ChatBody::Approval(shown) => Some(ToPhone::ApprovalClosed {
                    approval_id: shown.approval_id.clone(),
                    status: shown.status,
                }),
                _ => None,
            },
            Event::QuestionChanged(question) if question.status == QuestionStatus::Open => {
                cards::question_card(daemon, question).map(|card| ToPhone::QuestionOpen { card })
            }
            Event::QuestionChanged(question) => Some(ToPhone::QuestionClosed {
                question_id: question.id.clone(),
                status: question.status,
            }),
            _ => None,
        };
        let Some(message) = message else {
            return;
        };
        for phone in self.online_phones(daemon) {
            self.send_to(daemon, &phone.id, &message);
        }
        // The server may wake phones that are not on (spec 28.8).
        if matches!(
            message,
            ToPhone::ApprovalOpen { .. } | ToPhone::QuestionOpen { .. }
        ) {
            self.send_frame(&json!({ "t": "wake" }));
        }
    }

    /// Sends `phone` what is waiting right now.
    fn snapshot(&self, daemon: &Daemon, phone: &str) {
        let (approvals, questions) = cards::waiting(daemon);
        let mut room = SNAPSHOT_ROOM;
        let mut fits = |size: usize| {
            if size <= room {
                room -= size;
                true
            } else {
                false
            }
        };
        let (mut first_approvals, mut later_approvals) = (Vec::new(), Vec::new());
        for card in approvals {
            let size = serde_json::to_vec(&card).map_or(usize::MAX, |bytes| bytes.len());
            if later_approvals.is_empty() && fits(size) {
                first_approvals.push(card);
            } else {
                later_approvals.push(card);
            }
        }
        let (mut first_questions, mut later_questions) = (Vec::new(), Vec::new());
        for card in questions {
            let size = serde_json::to_vec(&card).map_or(usize::MAX, |bytes| bytes.len());
            if later_questions.is_empty() && fits(size) {
                first_questions.push(card);
            } else {
                later_questions.push(card);
            }
        }
        self.send_to(
            daemon,
            phone,
            &ToPhone::Snapshot {
                approvals: first_approvals,
                questions: first_questions,
            },
        );
        for card in later_approvals {
            self.send_to(daemon, phone, &ToPhone::ApprovalOpen { card });
        }
        for card in later_questions {
            self.send_to(daemon, phone, &ToPhone::QuestionOpen { card });
        }
    }

    /// A message from a phone came through the relay: opens it, and acts on
    /// it if it is new. The relay is told it was taken either way, so what
    /// does not open is not sent again.
    pub(crate) async fn receive(
        &self,
        daemon: &Arc<Daemon>,
        from: &str,
        frame_seq: u64,
        body: &str,
    ) {
        let ack = || self.send_frame(&json!({ "t": "ack", "from": from, "upto": frame_seq }));
        let Ok((_, credentials)) = identity(daemon) else {
            return;
        };
        let known = self.phones.get(&credentials.url, &credentials.device, from);
        let Some((phone, keys)) = known.and_then(|phone| phone.keys().map(|keys| (phone, keys)))
        else {
            ack();
            return;
        };
        let Ok((seq, plain)) = seal::open(&keys.p2c, Dir::PhoneToComputer, from, body) else {
            warn!("a message from a phone did not open and was dropped");
            ack();
            return;
        };
        // The same message again (the relay repeats what it was not told
        // about), or an old one sent back.
        if seq != frame_seq || seq <= phone.received {
            ack();
            return;
        }
        let now = daemon.clock.now_ms();
        let saved = self
            .phones
            .update(&credentials.url, &credentials.device, from, |phone| {
                phone.received = seq;
                phone.last_seen_at = Some(now);
            });
        if !matches!(saved, Ok(true)) {
            return;
        }
        ack();
        let Ok(message) = serde_json::from_slice::<FromPhone>(&plain) else {
            debug!("a phone sent something this version does not know");
            return;
        };
        self.apply(daemon, from, message).await;
    }

    async fn apply(&self, daemon: &Arc<Daemon>, phone: &str, message: FromPhone) {
        match message {
            FromPhone::Sync => self.snapshot(daemon, phone),
            FromPhone::ApprovalAnswer {
                approval_id,
                allow,
                note,
            } => {
                if self.may_answer(daemon, phone) {
                    self.answer_approval(daemon, phone, approval_id, allow, note)
                        .await;
                }
            }
            FromPhone::QuestionAnswer {
                question_id,
                answer,
            } => {
                if self.may_answer(daemon, phone) {
                    let params = QuestionsAnswerParams {
                        question_id: question_id.clone(),
                        answer,
                    };
                    let done = blocking(daemon, move |daemon| {
                        questions::answer(daemon, params).is_ok()
                    })
                    .await;
                    if !done {
                        self.say_question_is_closed(daemon, phone, &question_id);
                    }
                }
            }
            FromPhone::QuestionDismiss { question_id } => {
                if self.may_answer(daemon, phone) {
                    let params = QuestionIdParams {
                        question_id: question_id.clone(),
                    };
                    let done = blocking(daemon, move |daemon| {
                        questions::dismiss(daemon, params).is_ok()
                    })
                    .await;
                    if !done {
                        self.say_question_is_closed(daemon, phone, &question_id);
                    }
                }
            }
        }
    }

    async fn answer_approval(
        &self,
        daemon: &Arc<Daemon>,
        phone: &str,
        id: ApprovalId,
        allow: bool,
        note: Option<String>,
    ) {
        let Some((item, shown)) = cards::approval_item(daemon, &id) else {
            return;
        };
        if shown.status != ApprovalStatus::Pending {
            self.send_to(
                daemon,
                phone,
                &ToPhone::ApprovalClosed {
                    approval_id: id,
                    status: shown.status,
                },
            );
            return;
        }
        // The phone allows only what it showed whole and plain (28.5).
        if allow {
            let card = cards::approval_card(daemon, &item, &shown);
            if card
                .as_ref()
                .is_none_or(|card| card.cut || card.at_computer)
            {
                warn!("a phone tried to allow a request that is answered at the computer");
                if let Some(card) = card {
                    self.send_to(daemon, phone, &ToPhone::ApprovalOpen { card });
                }
                return;
            }
        }
        let params = ApprovalsAnswerParams {
            approval_id: id.clone(),
            allow,
            note: note.map(|note| note.chars().take(500).collect()),
            input: None,
            always: None,
        };
        let done = blocking(daemon, move |daemon| {
            approvals::answer(daemon, params).is_ok()
        })
        .await;
        if !done {
            let status = cards::approval_item(daemon, &id).map(|(_, shown)| shown.status);
            if let Some(status) = status.filter(|status| *status != ApprovalStatus::Pending) {
                self.send_to(
                    daemon,
                    phone,
                    &ToPhone::ApprovalClosed {
                        approval_id: id,
                        status,
                    },
                );
            }
        }
    }

    fn say_question_is_closed(&self, daemon: &Daemon, phone: &str, id: &QuestionId) {
        let status = daemon
            .store()
            .question(id)
            .ok()
            .flatten()
            .map(|record| record.question.status);
        if let Some(status) = status.filter(|status| *status != QuestionStatus::Open) {
            self.send_to(
                daemon,
                phone,
                &ToPhone::QuestionClosed {
                    question_id: id.clone(),
                    status,
                },
            );
        }
    }
}

/// Runs a service call where the store may be waited for.
async fn blocking(
    daemon: &Arc<Daemon>,
    work: impl FnOnce(&Daemon) -> bool + Send + 'static,
) -> bool {
    let daemon = Arc::clone(daemon);
    tokio::task::spawn_blocking(move || work(&daemon))
        .await
        .unwrap_or(false)
}
