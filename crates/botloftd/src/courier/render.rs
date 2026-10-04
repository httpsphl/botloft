//! The line a message becomes on the bot's stdin (spec 9.2 and 9.3).

use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::envelope::{self, Envelope, Sender};
use botloft_core::ids::random_uuid;
use botloft_core::protocol::{Attachment, Message, SenderKind, Task};
use bytes::Bytes;
use serde_json::{Value, json};
use tracing::debug;

use super::context::RoutineContext;

/// Images the API accepts inline (spec 9.5).
const INLINE_IMAGES: &[&str] = &["image/png", "image/jpeg", "image/gif", "image/webp"];
const INLINE_IMAGE_MAX: u64 = 5 * 1024 * 1024;

/// Who and where a message comes from, for the envelope.
pub(super) struct Context<'a> {
    pub crew_name: &'a str,
    /// Handle of the sending bot, for bot messages.
    pub sender_handle: Option<&'a str>,
    pub task: Option<&'a Task>,
    /// The recipient's workspace, where attachments were saved.
    pub workspace: &'a Path,
    /// For a routine's message: the routine and what made it run (spec
    /// 20.5, 20.13).
    pub routine: Option<&'a RoutineContext>,
    /// For the owner's answer to a question: its id and text (spec 23.4).
    pub question: Option<(&'a str, &'a str)>,
}

pub(super) struct Rendered {
    /// One stream-json line, with its newline.
    pub line: Bytes,
    /// Goes with the line; Claude Code gives it back when the turn begins.
    pub uuid: String,
}

pub(super) fn render(message: &Message, context: &Context<'_>, now: i64) -> Rendered {
    let text = match message.from_kind {
        // The owner is the session's user: their words go as written.
        SenderKind::Owner => {
            let body = with_attachment_list(&message.body, &message.attachments);
            let body = match &message.reply_to {
                Some(reply) => envelope::reply(&reply.text, &body),
                None => body,
            };
            match context.question {
                Some((id, question)) => envelope::answer(id, question, &body),
                None => body,
            }
        }
        SenderKind::Bot => {
            let from = match context.sender_handle {
                Some(handle) => Sender::Bot { handle },
                // Deleted while the message waited (spec 7.6).
                None => Sender::DeletedBot,
            };
            envelope(message, context, from, now)
        }
        SenderKind::System => match context.routine {
            Some(routine) => routine.envelope(&message.body).render(),
            None => envelope(message, context, Sender::Botloft, now),
        },
    };
    let mut content = vec![json!({ "type": "text", "text": text })];
    content.extend(
        message
            .attachments
            .iter()
            .filter_map(|attachment| inline_image(attachment, context.workspace)),
    );
    let uuid = random_uuid();
    let event = json!({
        "type": "user",
        "uuid": uuid,
        "message": { "role": "user", "content": content },
    });
    let mut line = event.to_string().into_bytes();
    line.push(b'\n');
    Rendered {
        line: Bytes::from(line),
        uuid,
    }
}

fn envelope(message: &Message, context: &Context<'_>, from: Sender<'_>, now: i64) -> String {
    Envelope {
        from,
        crew: context.crew_name,
        kind: message.kind,
        task: context.task,
        body: &message.body,
    }
    .render(now)
}

/// The body, then where each attachment was saved (spec 9.5).
fn with_attachment_list(body: &str, attachments: &[Attachment]) -> String {
    if attachments.is_empty() {
        return body.to_owned();
    }
    let paths: Vec<&str> = attachments.iter().map(|a| a.path.as_str()).collect();
    format!(
        "{body}\n\nAttached files, saved in your folder: {}",
        paths.join(", ")
    )
}

/// An image block for a small enough image; `None` for anything else or
/// a file that cannot be read.
fn inline_image(attachment: &Attachment, workspace: &Path) -> Option<Value> {
    if !INLINE_IMAGES.contains(&attachment.media_type.as_str())
        || attachment.size > INLINE_IMAGE_MAX
    {
        return None;
    }
    match std::fs::read(workspace.join(&attachment.path)) {
        Ok(bytes) => Some(json!({
            "type": "image",
            "source": {
                "type": "base64",
                "media_type": attachment.media_type,
                "data": BASE64.encode(bytes),
            },
        })),
        Err(err) => {
            debug!(attachment = %attachment.id, "attachment not inlined: {err}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::ids::{AttachmentId, BotId, ChatItemId, CrewId, MessageId};
    use botloft_core::protocol::{MessageKind, MessageReply};

    use super::*;

    fn message(from_kind: SenderKind, body: &str) -> Message {
        Message {
            id: MessageId::generate(),
            crew_id: CrewId::generate(),
            from_kind,
            from_bot_id: None,
            to_bot_id: BotId::generate(),
            kind: MessageKind::Note,
            body: body.into(),
            task_id: None,
            routine_id: None,
            question_id: None,
            attachments: Vec::new(),
            reply_to: None,
            created_at: 0,
        }
    }

    fn parse(rendered: &Rendered) -> Value {
        assert!(rendered.line.ends_with(b"\n"));
        serde_json::from_slice(&rendered.line).expect("json line")
    }

    #[test]
    fn the_owner_speaks_as_the_user_and_bots_come_in_an_envelope() {
        let dir = tempfile::tempdir().expect("tempdir");
        let context = Context {
            crew_name: "Ops",
            sender_handle: Some("scout"),
            task: None,
            workspace: dir.path(),
            routine: None,
            question: None,
        };
        let owner = render(&message(SenderKind::Owner, "Ship it"), &context, 0);
        let line = parse(&owner);
        assert_eq!(line["type"], "user");
        assert_eq!(line["uuid"], owner.uuid);
        assert_eq!(line["message"]["content"][0]["text"], "Ship it");

        let bot = parse(&render(&message(SenderKind::Bot, "hi"), &context, 0));
        let text = bot["message"]["content"][0]["text"].as_str().expect("text");
        assert!(
            text.starts_with("[botloft] from @scout · crew Ops"),
            "{text}"
        );
        assert!(text.ends_with("hi"));
    }

    #[test]
    fn a_reply_quotes_the_chat_item_above_the_owner_words() {
        let dir = tempfile::tempdir().expect("tempdir");
        let context = Context {
            crew_name: "Ops",
            sender_handle: None,
            task: None,
            workspace: dir.path(),
            routine: None,
            question: None,
        };
        let mut owner = message(SenderKind::Owner, "Make it yearly");
        owner.reply_to = Some(MessageReply {
            item_id: ChatItemId::generate(),
            text: "Acme signs monthly.".into(),
        });
        let line = parse(&render(&owner, &context, 0));
        assert_eq!(
            line["message"]["content"][0]["text"],
            "Replying to: \"Acme signs monthly.\"

Make it yearly"
        );
    }

    #[test]
    fn attachments_are_listed_and_small_images_go_inline() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("attachments")).expect("dir");
        std::fs::write(dir.path().join("attachments/shot.png"), b"png bytes").expect("png");
        let mut owner = message(SenderKind::Owner, "Look");
        owner.attachments = vec![
            Attachment {
                id: AttachmentId::generate(),
                name: "shot.png".into(),
                media_type: "image/png".into(),
                size: 9,
                path: "attachments/shot.png".into(),
            },
            Attachment {
                id: AttachmentId::generate(),
                name: "notes.pdf".into(),
                media_type: "application/pdf".into(),
                size: 100,
                path: "attachments/notes.pdf".into(),
            },
        ];
        let context = Context {
            crew_name: "Ops",
            sender_handle: None,
            task: None,
            workspace: dir.path(),
            routine: None,
            question: None,
        };
        let line = parse(&render(&owner, &context, 0));
        let content = line["message"]["content"].as_array().expect("blocks");
        assert_eq!(content.len(), 2, "text and the one image");
        assert_eq!(
            content[0]["text"],
            "Look\n\nAttached files, saved in your folder: attachments/shot.png, attachments/notes.pdf"
        );
        assert_eq!(content[1]["type"], "image");
        assert_eq!(content[1]["source"]["media_type"], "image/png");
        assert_eq!(content[1]["source"]["data"], BASE64.encode(b"png bytes"));
    }
}
