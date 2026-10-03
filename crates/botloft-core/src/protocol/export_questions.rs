//! The questions part of the TypeScript bindings (spec 23).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

/// Before `ChatBody`, which holds `QuestionItem`.
pub(super) fn decls(out: &mut Out) {
    out.decl::<QuestionStatus>();
    out.decl::<Question>();
    out.decl::<QuestionItem>();
    out.decl::<QuestionsListParams>();
    out.decl::<QuestionsAnswerParams>();
    out.decl::<QuestionIdParams>();
}

pub(super) fn methods(out: &mut Out) {
    let question = out.name::<Question>();
    out.method(
        method::QUESTIONS_LIST,
        &out.name::<QuestionsListParams>(),
        &out.name::<Vec<Question>>(),
    );
    out.method(
        method::QUESTIONS_ANSWER,
        &out.name::<QuestionsAnswerParams>(),
        &question,
    );
    out.method(
        method::QUESTIONS_DISMISS,
        &out.name::<QuestionIdParams>(),
        &question,
    );
}

pub(super) fn notifications(out: &mut Out) {
    let question = out.name::<Question>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {question};",
        notification::QUESTION_CHANGED
    );
}
