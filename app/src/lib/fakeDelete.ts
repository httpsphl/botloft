// What the fake daemon forgets when a bot is deleted for good (spec 7.6):
// its chat, its routines, its tasks and what it received. What it sent to
// other bots stays, without a sender.

import type { FakeBotloft } from "./fake";
import type { Bot } from "./protocol.gen";

/** Drops every item that fails `keep`, in place. */
function retain<T>(list: T[], keep: (item: T) => boolean): void {
  list.splice(0, list.length, ...list.filter(keep));
}

/** Removes the bot and everything that was its own. Sends no notification. */
export function purgeBot(fake: FakeBotloft, bot: Bot): void {
  const { chat, conversation, routines } = fake;
  fake.bots.delete(bot.id);
  retain(chat.items, (item) => item.botId !== bot.id);
  for (const [id, approval] of chat.approvals) {
    if (approval.botId === bot.id) {
      chat.approvals.delete(id);
    }
  }
  for (const [id, routine] of routines.routines) {
    if (routine.botId === bot.id) {
      routines.routines.delete(id);
    }
  }
  for (const [id, delivery] of conversation.deliveries) {
    if (delivery.botId === bot.id) {
      conversation.deliveries.delete(id);
    }
  }
  const gone = new Set<string>();
  for (const [id, task] of conversation.tasks) {
    if (task.requesterBotId === bot.id || task.assigneeBotId === bot.id) {
      conversation.tasks.delete(id);
      gone.add(id);
    }
  }
  retain(conversation.messages, (message) => message.toBotId !== bot.id);
  for (const message of conversation.messages) {
    if (message.fromBotId === bot.id) {
      message.fromBotId = null;
    }
    if (message.taskId !== null && gone.has(message.taskId)) {
      message.taskId = null;
    }
  }
}
