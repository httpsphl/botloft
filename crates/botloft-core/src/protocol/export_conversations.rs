//! The conversations part of the TypeScript bindings (spec 40).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

/// After `Bot`: a conversation holds a context and background tasks.
pub(super) fn decls(out: &mut Out) {
    out.decl::<TitleBy>();
    out.decl::<Conversation>();
    out.decl::<ConversationsListParams>();
    out.decl::<ConversationIdParams>();
    out.decl::<ConversationsRenameParams>();
    out.decl::<BotConversationParams>();
}

pub(super) fn methods(out: &mut Out) {
    let one = out.name::<Conversation>();
    let many = out.name::<Vec<Conversation>>();
    let bot = out.name::<BotIdParams>();
    let id = out.name::<ConversationIdParams>();
    out.method(
        method::CONVERSATIONS_LIST,
        &out.name::<ConversationsListParams>(),
        &many,
    );
    out.method(method::CONVERSATIONS_CREATE, &bot, &one);
    out.method(
        method::CONVERSATIONS_RENAME,
        &out.name::<ConversationsRenameParams>(),
        &one,
    );
    out.method(method::CONVERSATIONS_ARCHIVE, &id, &one);
    out.method(method::CONVERSATIONS_UNARCHIVE, &id, &one);
    out.method(method::CONVERSATIONS_ARCHIVED, &bot, &many);
}

pub(super) fn notifications(out: &mut Out) {
    let one = out.name::<Conversation>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {one};",
        notification::CONVERSATION_CHANGED
    );
}
