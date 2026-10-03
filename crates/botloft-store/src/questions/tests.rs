//! Tests for the questions bots ask the owner.

use botloft_core::ids::{BotId, ChatItemId, QuestionId};
use botloft_core::protocol::{ChatBody, ChatItem, Question, QuestionItem, QuestionStatus};

use crate::Store;
use crate::tests::{Fixture, message_to};

/// Asks `text` as `bot`; returns the question.
pub(crate) fn ask(store: &Store, fx: &Fixture, bot: &BotId, text: &str, at: i64) -> Question {
    let question = Question {
        id: QuestionId::generate(),
        crew_id: fx.crew.id.clone(),
        bot_id: bot.clone(),
        text: text.into(),
        options: vec!["Yes".into(), "No".into()],
        status: QuestionStatus::Open,
        answer: None,
        created_at: at,
        answered_at: None,
    };
    let item = ChatItem {
        id: ChatItemId::generate(),
        bot_id: bot.clone(),
        body: ChatBody::Question(QuestionItem {
            question: question.clone(),
        }),
        created_at: at,
        updated_at: at,
    };
    store.insert_question(&question, &item).expect("question");
    question
}

#[test]
fn open_questions_are_listed_newest_first_and_counted_per_bot() {
    let fx = Fixture::new();
    let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
    let first = ask(&fx.store, &fx, scout, "Which client?", 10);
    let second = ask(&fx.store, &fx, writer, "Which tone?", 20);

    let open = fx.store.questions(QuestionStatus::Open).expect("list");
    assert_eq!(
        open.iter().map(|q| &q.id).collect::<Vec<_>>(),
        [&second.id, &first.id]
    );
    assert_eq!(open[1].options, ["Yes", "No"]);
    assert_eq!(fx.store.open_questions(scout).expect("count"), 1);

    let record = fx.store.question(&first.id).expect("find").expect("there");
    let item = fx
        .store
        .chat_item(&record.chat_item_id)
        .expect("item")
        .expect("there");
    assert!(matches!(item.body, ChatBody::Question(_)));
}

#[test]
fn an_answer_closes_the_question_and_becomes_a_message_once() {
    let fx = Fixture::new();
    let scout = &fx.bots[0].id;
    let question = ask(&fx.store, &fx, scout, "Ship on Friday?", 10);
    let (mut message, delivery) = message_to(&fx.crew.id, scout, "Yes");
    message.question_id = Some(question.id.clone());
    message.created_at = 50;

    let answered = fx
        .store
        .answer_question(&question.id, &message, &delivery)
        .expect("answer")
        .expect("was open");
    assert_eq!(answered.question.status, QuestionStatus::Answered);
    assert_eq!(answered.question.answer.as_deref(), Some("Yes"));
    assert_eq!(answered.question.answered_at, Some(50));
    let ChatBody::Question(item) = &answered.question_item.body else {
        panic!("question item");
    };
    assert_eq!(item.question, answered.question);
    let ChatBody::Inbound(inbound) = &answered.answer_item.body else {
        panic!("answer item");
    };
    assert_eq!(inbound.message.question_id.as_ref(), Some(&question.id));
    let stored = fx.store.message(&message.id).expect("read").expect("saved");
    assert_eq!(stored.question_id, Some(question.id.clone()));

    let (again, delivery) = message_to(&fx.crew.id, scout, "No");
    assert_eq!(
        fx.store
            .answer_question(&question.id, &again, &delivery)
            .expect("again"),
        None
    );
    assert_eq!(fx.store.message(&again.id).expect("read"), None);
    assert_eq!(
        fx.store
            .dismiss_question(&question.id, 60)
            .expect("dismiss"),
        None
    );
    assert!(
        fx.store
            .questions(QuestionStatus::Open)
            .expect("list")
            .is_empty()
    );
    assert_eq!(fx.store.open_questions(scout).expect("count"), 0);
}

#[test]
fn a_dismissed_question_leaves_the_box_and_archived_bots_hide_theirs() {
    let fx = Fixture::new();
    let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
    let dismissed = ask(&fx.store, &fx, scout, "Coffee?", 10);
    ask(&fx.store, &fx, writer, "Tea?", 20);

    let (question, item) = fx
        .store
        .dismiss_question(&dismissed.id, 30)
        .expect("dismiss")
        .expect("was open");
    assert_eq!(question.status, QuestionStatus::Dismissed);
    assert_eq!(question.answer, None);
    assert_eq!(item.updated_at, 30);
    assert_eq!(
        fx.store
            .questions(QuestionStatus::Dismissed)
            .expect("list")
            .len(),
        1
    );

    let mut archived = fx.bots[1].clone();
    archived.archived_at = Some(40);
    fx.store.update_bot(&archived).expect("archive");
    assert!(
        fx.store
            .questions(QuestionStatus::Open)
            .expect("list")
            .is_empty()
    );
}
