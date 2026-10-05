//! What changed, as the notification the apps get (spec 11).

use botloft_core::protocol::notification;

use super::jsonrpc;
use crate::state::Event;

pub(super) fn to_notification(event: &Event) -> String {
    use jsonrpc::notification as note;
    match event {
        Event::CrewChanged(crew) => note(notification::CREW_CHANGED, crew),
        Event::CrewDeleted(crew) => note(notification::CREW_DELETED, crew),
        Event::BotChanged(bot) => note(notification::BOT_CHANGED, bot),
        Event::BotDeleted(bot) => note(notification::BOT_DELETED, bot),
        Event::FolderRecycled(folder) => note(notification::FOLDER_RECYCLED, folder),
        Event::BotState(state) => note(notification::BOT_STATE, state),
        Event::BotContext(context) => note(notification::BOT_CONTEXT, context),
        Event::BotRules(rules) => note(notification::BOT_RULES, rules),
        Event::BotDesktop(desktop) => note(notification::BOT_DESKTOP, desktop),
        Event::DesktopChanged(state) => note(notification::DESKTOP_CHANGED, state),
        Event::DesktopAway(uses) => note(notification::DESKTOP_AWAY, uses),
        Event::ChatItem(item) => note(notification::CHAT_ITEM, item),
        Event::ChatDelta(delta) => note(notification::CHAT_DELTA, delta),
        Event::MessageCreated(message) => note(notification::MESSAGE_CREATED, message),
        Event::DeliveryChanged(delivery) => note(notification::DELIVERY_CHANGED, delivery),
        Event::TaskChanged(task) => note(notification::TASK_CHANGED, task),
        Event::RoutineChanged(routine) => note(notification::ROUTINE_CHANGED, routine),
        Event::RoutineRun(run) => note(notification::ROUTINE_RUN, run),
        Event::BrowserChanged(state) => note(notification::BROWSER_CHANGED, state),
        Event::BrowserAction(action) => note(notification::BROWSER_ACTION, action),
        Event::ScreenDraft(draft) => note(notification::SCREEN_DRAFT, draft),
        Event::QuestionChanged(question) => note(notification::QUESTION_CHANGED, question),
        Event::ReactionChanged(change) => note(notification::REACTION_CHANGED, change),
    }
}
