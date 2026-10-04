//! Teaching a bot a task (spec 21.13): while the owner has the browser in
//! their hands and teaches, each thing they do becomes a step by what it
//! means, read from the page before it reaches it: the button clicked, the
//! field typed in, the address opened. What they type is never read.

use std::sync::Arc;

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    BrowserInput, LESSON_STEPS_MAX, LessonStep, LessonStepKind, MouseAction, MouseButton, modifier,
};
use serde_json::Value;
use tokio::sync::broadcast;

use super::moves::Move;
use super::session::Session;
use super::{Slots, lock, update};
use crate::clock::Clock;
use crate::state::Event;

/// Longest label a step keeps, in characters.
const LABEL_MAX: usize = 80;

/// Where a lesson's steps go: the bot's browser state, which the app
/// follows.
pub(super) struct Teacher {
    pub slots: Slots,
    pub events: broadcast::Sender<Event>,
    pub clock: Arc<dyn Clock>,
    pub bot: BotId,
}

impl Teacher {
    fn teaching(&self) -> bool {
        lock(&self.slots)
            .get(&self.bot)
            .is_some_and(|slot| slot.state.lesson.is_some())
    }

    /// Adds a step to the lesson going on, if one is. Typing on in the
    /// field of the step before is the same step.
    fn add(&self, step: LessonStep) {
        update(
            &self.slots,
            &self.events,
            self.clock.as_ref(),
            &self.bot,
            |state| {
                let Some(steps) = state.lesson.as_mut() else {
                    return;
                };
                let same_field = |last: &LessonStep| {
                    last.kind == LessonStepKind::Type
                        && step.kind == LessonStepKind::Type
                        && last.label == step.label
                };
                if steps.last().is_some_and(same_field) || steps.len() >= LESSON_STEPS_MAX {
                    return;
                }
                steps.push(step);
            },
        );
    }

    /// What `step` adds to the lesson, read from the page before the step
    /// reaches it.
    pub(super) async fn observe(&self, session: &Session, step: &Move) {
        if !self.teaching() {
            return;
        }
        let found = match step {
            Move::Open(url) => address(url).map(|url| plain(LessonStepKind::Open, url)),
            Move::Input(BrowserInput::Mouse {
                action: MouseAction::Down,
                button: MouseButton::Left,
                clicks,
                x,
                y,
                ..
            }) if *clicks <= 1 => {
                read(session, &format!("at({x}, {y})"), LessonStepKind::Click).await
            }
            Move::Input(BrowserInput::Key { key, modifiers, .. }) => {
                let plain_key = modifiers & (modifier::CTRL | modifier::ALT | modifier::META) == 0;
                if plain_key && matches!(key.as_str(), "Enter" | "Escape" | "Tab") {
                    Some(plain(LessonStepKind::Press, key.clone()))
                } else if plain_key && (key.chars().count() == 1 || key == "Backspace") {
                    read(session, "focused()", LessonStepKind::Type).await
                } else {
                    None
                }
            }
            Move::Input(BrowserInput::Text { .. }) => {
                read(session, "focused()", LessonStepKind::Type).await
            }
            _ => None,
        };
        if let Some(found) = found {
            self.add(found);
        }
    }
}

fn plain(kind: LessonStepKind, label: String) -> LessonStep {
    LessonStep {
        kind,
        label,
        role: None,
        secret: false,
    }
}

/// The element the page script found, as a step; `None` when nothing with
/// a role is there.
async fn read(session: &Session, call: &str, kind: LessonStepKind) -> Option<LessonStep> {
    let found = session.reader(call).await.ok()?;
    meaning(&found, kind)
}

fn meaning(found: &Value, kind: LessonStepKind) -> Option<LessonStep> {
    if found["none"] == true {
        return None;
    }
    let label: String = found["label"].as_str()?.chars().take(LABEL_MAX).collect();
    Some(LessonStep {
        kind,
        label,
        role: found["role"].as_str().map(str::to_owned),
        secret: found["secret"] == true,
    })
}

/// A web address as a lesson keeps it: its site and path, without the
/// query or the fragment, where sign-in tokens and personal data travel.
/// `None` for anything but `http` and `https`.
pub(super) fn address(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return None;
    }
    let rest = rest.split(['?', '#']).next()?;
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    // Never a name and password written into the address.
    let host = authority.rsplit('@').next()?;
    if host.is_empty() {
        return None;
    }
    let kept = format!("{}://{host}/{path}", scheme.to_ascii_lowercase());
    Some(kept.chars().take(LABEL_MAX * 2).collect())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_address_keeps_its_site_and_path_only() {
        assert_eq!(
            address("https://mail.example.com/inbox?token=secret#msg-1").as_deref(),
            Some("https://mail.example.com/inbox")
        );
        assert_eq!(
            address("https://ana:hunter2@example.com").as_deref(),
            Some("https://example.com/")
        );
        assert_eq!(address("about:blank"), None);
        assert_eq!(address("file:///C:/notes.txt"), None);
    }

    #[test]
    fn what_the_page_found_becomes_a_step_never_a_value() {
        let field = json!({ "role": "textbox", "label": "Password", "secret": true });
        let step = meaning(&field, LessonStepKind::Type).expect("step");
        assert_eq!(step.label, "Password");
        assert_eq!(step.role.as_deref(), Some("textbox"));
        assert!(step.secret);
        assert_eq!(
            meaning(&json!({ "none": true }), LessonStepKind::Click),
            None
        );
    }
}
