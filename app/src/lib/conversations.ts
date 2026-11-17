// The conversations of a bot (spec 40). The main one has the bot's own
// ULID: anything that knows the bot knows it.

import type { BotId, ConversationId } from "./protocol.gen";

/** The main conversation of `botId`. */
export function mainConversationOf(botId: BotId): ConversationId {
  return `cnv_${botId.slice("bot_".length)}`;
}
