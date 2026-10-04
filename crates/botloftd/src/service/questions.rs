//! Questions a bot asks the owner without waiting (spec 23): asking, from
//! the bot's tool, and `questions.*`, from the app.

use botloft_core::ids::{BotId, ChatItemId, MessageId, QuestionId};
use botloft_core::protocol::{
    ChatBody, ChatItem, Message, MessageKind, OPEN_QUESTIONS_MAX, QUESTION_MAX_CHARS,
    QUESTION_OPTION_MAX_CHARS, QUESTION_OPTIONS_MAX, QUESTION_OPTIONS_MIN, Question,
    QuestionIdParams, QuestionItem, QuestionStatus, QuestionsAnswerParams, QuestionsListParams,
    SenderKind,
};
use botloft_core::validate;
use botloft_store::Store;

use super::{ApiError, ApiResult, bots, messages};
use crate::chat::items;
use crate::state::{Daemon, Event};

/// What a bot asks: the text and the ready answers, if any.
pub struct Ask {
    pub text: String,
    pub options: Vec<String>,
}

/// Checks and keeps a bot's question, and shows it in its chat and the
/// owner's box.
pub fn ask(daemon: &Daemon, bot: &BotId, ask: Ask) -> ApiResult<Question> {
    let text = ask.text.trim();
    if text.is_empty() {
        return Err(ApiError::validation("question must not be empty"));
    }
    if text.chars().count() > QUESTION_MAX_CHARS {
        return Err(ApiError::validation(format!(
            "question must be at most {QUESTION_MAX_CHARS} characters"
        )));
    }
    let options = options(ask.options)?;
    let store = daemon.store();
    let (crew, record) = bots::active(&store, bot)?;
    if store.open_questions(bot)? >= OPEN_QUESTIONS_MAX {
        return Err(ApiError::Conflict(format!(
            "you already have {OPEN_QUESTIONS_MAX} questions the owner has not answered; wait \
             for an answer before asking another"
        )));
    }
    let now = daemon.clock.now_ms();
    let question = Question {
        id: QuestionId::generate(),
        crew_id: crew.id,
        bot_id: record.id,
        text: text.to_owned(),
        options,
        status: QuestionStatus::Open,
        answer: None,
        created_at: now,
        answered_at: None,
    };
    let item = ChatItem {
        id: ChatItemId::generate(),
        bot_id: bot.clone(),
        body: ChatBody::Question(QuestionItem {
            question: question.clone(),
        }),
        created_at: now,
        updated_at: now,
    };
    store.insert_question(&question, &item)?;
    drop(store);
    items::announce(daemon, item);
    daemon.emit(Event::QuestionChanged(question.clone()));
    Ok(question)
}

/// None, or 2 to 5 distinct one-line answers.
fn options(options: Vec<String>) -> ApiResult<Vec<String>> {
    if options.is_empty() {
        return Ok(options);
    }
    if !(QUESTION_OPTIONS_MIN..=QUESTION_OPTIONS_MAX).contains(&options.len()) {
        return Err(ApiError::validation(format!(
            "options takes {QUESTION_OPTIONS_MIN} to {QUESTION_OPTIONS_MAX} answers, or none"
        )));
    }
    let mut kept: Vec<String> = Vec::with_capacity(options.len());
    for option in options {
        let option = option.trim();
        if option.is_empty() {
            return Err(ApiError::validation("options must not be empty"));
        }
        if option.chars().count() > QUESTION_OPTION_MAX_CHARS
            || option.chars().any(char::is_control)
        {
            return Err(ApiError::validation(format!(
                "each option must be one line of at most {QUESTION_OPTION_MAX_CHARS} characters"
            )));
        }
        if kept.iter().any(|seen| seen.eq_ignore_ascii_case(option)) {
            return Err(ApiError::validation(format!(
                "the option \"{option}\" is repeated"
            )));
        }
        kept.push(option.to_owned());
    }
    Ok(kept)
}

pub fn list(daemon: &Daemon, params: QuestionsListParams) -> ApiResult<Vec<Question>> {
    let status = params.status.unwrap_or(QuestionStatus::Open);
    Ok(daemon.store().questions(status)?)
}

/// The owner answers: the question closes and the answer goes to the bot
/// as the owner's message.
pub fn answer(daemon: &Daemon, params: QuestionsAnswerParams) -> ApiResult<Question> {
    let body = validate::message("answer", &params.answer)?;
    let store = daemon.store();
    let question = open(&store, &params.question_id)?;
    let (crew, bot) = bots::active(&store, &question.bot_id)?;
    let message = Message {
        id: MessageId::generate(),
        crew_id: crew.id,
        from_kind: SenderKind::Owner,
        from_bot_id: None,
        to_bot_id: bot.id,
        kind: MessageKind::Note,
        body,
        task_id: None,
        routine_id: None,
        question_id: Some(question.id.clone()),
        reply_to: None,
        attachments: Vec::new(),
        created_at: daemon.clock.now_ms(),
    };
    let delivery = messages::pending_delivery(&message);
    let answered = store
        .answer_question(&question.id, &message, &delivery)?
        .ok_or_else(|| already_closed(&question.id))?;
    drop(store);
    items::changed(daemon, answered.question_item);
    messages::announce(daemon, None, &message, delivery, answered.answer_item);
    daemon.emit(Event::QuestionChanged(answered.question.clone()));
    Ok(answered.question)
}

/// The owner closes a question without answering; the bot is not told.
pub fn dismiss(daemon: &Daemon, params: QuestionIdParams) -> ApiResult<Question> {
    let store = daemon.store();
    let question = open(&store, &params.question_id)?;
    bots::active(&store, &question.bot_id)?;
    let (question, item) = store
        .dismiss_question(&question.id, daemon.clock.now_ms())?
        .ok_or_else(|| already_closed(&question.id))?;
    drop(store);
    items::changed(daemon, item);
    daemon.emit(Event::QuestionChanged(question.clone()));
    Ok(question)
}

/// The question, if it is still open.
fn open(store: &Store, id: &QuestionId) -> ApiResult<Question> {
    let record = store
        .question(id)?
        .ok_or_else(|| ApiError::NotFound(format!("question {id} does not exist")))?;
    if record.question.status != QuestionStatus::Open {
        return Err(already_closed(id));
    }
    Ok(record.question)
}

fn already_closed(id: &QuestionId) -> ApiError {
    ApiError::Conflict(format!("question {id} was already answered or dismissed"))
}
