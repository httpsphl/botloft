use botloft_core::protocol::{MessageKind, SenderKind};

use super::*;
use crate::DeliveryOutcome;
use crate::tests::{Fixture, message_to};

#[test]
fn a_message_is_saved_with_its_delivery_and_read_back() {
    let fx = Fixture::new();
    let (message, delivery) = message_to(&fx.crew.id, &fx.bots[0].id, "hello");
    fx.store
        .insert_message(&message, &delivery, None)
        .expect("insert");
    assert_eq!(
        fx.store.message(&message.id).expect("read"),
        Some(message.clone())
    );
    assert_eq!(
        fx.store.delivery(&delivery.id).expect("read"),
        Some(delivery)
    );
    assert_eq!(message.from_kind, SenderKind::Owner);
    assert_eq!(message.kind, MessageKind::Note);
}

#[test]
fn attachments_and_the_chat_item_come_with_the_message() {
    let fx = Fixture::new();
    let (mut message, delivery) = message_to(&fx.crew.id, &fx.bots[0].id, "see attached");
    message.attachments = vec![Attachment {
        id: botloft_core::ids::AttachmentId::generate(),
        name: "plan.pdf".into(),
        media_type: "application/pdf".into(),
        size: 2048,
        path: "attachments/2026-09-28/plan.pdf".into(),
    }];
    let item = fx
        .store
        .insert_message(&message, &delivery, None)
        .expect("insert");
    assert_eq!(item.bot_id, fx.bots[0].id);
    assert_eq!(
        item.body,
        ChatBody::Inbound(InboundItem {
            message: message.clone()
        })
    );
    assert_eq!(
        fx.store.message(&message.id).expect("read"),
        Some(message.clone())
    );
    let history = fx
        .store
        .chat_history(&fx.bots[0].id, None, 10)
        .expect("history");
    assert_eq!(history, vec![item]);
    let listed = fx
        .store
        .messages(MessageFilter {
            limit: 5,
            ..MessageFilter::default()
        })
        .expect("list");
    assert_eq!(listed[0].attachments, message.attachments);
    let first = &message.attachments[0];
    assert_eq!(
        fx.store.attachment(&first.id).expect("read"),
        Some((first.clone(), fx.bots[0].id.clone()))
    );
    let missing = botloft_core::ids::AttachmentId::generate();
    assert_eq!(fx.store.attachment(&missing).expect("read"), None);
}

#[test]
fn a_note_from_botloft_counts_until_the_bot_begins_it() {
    let fx = Fixture::new();
    let bot = &fx.bots[0].id;
    let note = |body: &str| {
        let (mut message, delivery) = message_to(&fx.crew.id, bot, body);
        message.from_kind = SenderKind::System;
        message.kind = MessageKind::System;
        fx.store
            .insert_message(&message, &delivery, None)
            .expect("insert");
        delivery
    };
    assert!(!fx.store.unread_note(bot, "go on").expect("query"));
    // The owner's words are not Botloft's note.
    let (owner, delivery) = message_to(&fx.crew.id, bot, "go on");
    fx.store
        .insert_message(&owner, &delivery, None)
        .expect("insert");
    assert!(!fx.store.unread_note(bot, "go on").expect("query"));

    let waiting = note("go on");
    assert!(fx.store.unread_note(bot, "go on").expect("query"));
    assert!(!fx.store.unread_note(bot, "other").expect("query"));
    assert!(
        !fx.store
            .unread_note(&fx.bots[1].id, "go on")
            .expect("query")
    );
    // Written to the bot, it still counts until the bot begins it.
    let sent = DeliveryOutcome::Sent {
        generation: 1,
        turn_uuid: "t-1",
    };
    fx.store
        .finish_delivery(&waiting.id, sent, 1)
        .expect("sent");
    assert!(fx.store.unread_note(bot, "go on").expect("query"));
    fx.store.mark_read("t-1", 2).expect("read");
    assert!(!fx.store.unread_note(bot, "go on").expect("query"));

    let given_up = note("go on");
    let dead = DeliveryOutcome::Dead { error: "gone" };
    fx.store
        .finish_delivery(&given_up.id, dead, 3)
        .expect("dead");
    assert!(!fx.store.unread_note(bot, "go on").expect("query"));
}

#[test]
fn a_failed_insert_leaves_nothing_behind() {
    let fx = Fixture::new();
    let (message, mut delivery) = message_to(&fx.crew.id, &fx.bots[0].id, "hello");
    delivery.bot_id = BotId::generate(); // no such bot: the FK fails
    assert!(fx.store.insert_message(&message, &delivery, None).is_err());
    assert_eq!(fx.store.message(&message.id).expect("read"), None);
}

#[test]
fn listing_filters_by_bot_and_pages_back_newest_first() {
    let fx = Fixture::new();
    let mut ids = Vec::new();
    for (i, bot) in [&fx.bots[0], &fx.bots[1], &fx.bots[0]].iter().enumerate() {
        let (message, delivery) = message_to(&fx.crew.id, &bot.id, &format!("m{i}"));
        fx.store
            .insert_message(&message, &delivery, None)
            .expect("insert");
        ids.push(message.id);
    }
    let all = fx
        .store
        .messages(MessageFilter {
            limit: 10,
            ..MessageFilter::default()
        })
        .expect("list");
    let bodies: Vec<_> = all.iter().map(|m| m.body.as_str()).collect();
    assert_eq!(bodies, ["m2", "m1", "m0"]);

    let first_bot = fx
        .store
        .messages(MessageFilter {
            bot: Some(&fx.bots[0].id),
            limit: 10,
            ..MessageFilter::default()
        })
        .expect("list");
    assert_eq!(first_bot.len(), 2);

    let older = fx
        .store
        .messages(MessageFilter {
            crew: Some(&fx.crew.id),
            before: Some(&ids[2]),
            limit: 1,
            ..MessageFilter::default()
        })
        .expect("page");
    assert_eq!(older.len(), 1);
    assert_eq!(older[0].body, "m1");
}
