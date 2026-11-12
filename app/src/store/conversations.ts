// The conversations of each agent (spec 40.7): the list of one is read
// when its chat opens, then kept current by `conversation.changed`. Each
// agent opens on the conversation the owner left it on, remembered across
// restarts of the app.

import type { StoreApi } from "zustand/vanilla";
import type { BotloftApi } from "../lib/api";
import { mainConversationOf } from "../lib/conversations";
import type { BotId, Conversation, ConversationId } from "../lib/protocol.gen";
import type { AppState } from "./app";

const OPEN_KEY = "botloft.conversations";

export function loadOpen(): Record<BotId, ConversationId> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(OPEN_KEY) ?? "{}");
    return typeof parsed === "object" && parsed !== null
      ? (parsed as Record<BotId, ConversationId>)
      : {};
  } catch {
    return {};
  }
}

function saveOpen(open: Record<BotId, ConversationId>): void {
  try {
    localStorage.setItem(OPEN_KEY, JSON.stringify(open));
  } catch {
    // Storage may be off: each agent opens on its main conversation.
  }
}

/** The main one first, then the last one used. */
function ordered(list: Conversation[]): Conversation[] {
  return [...list].sort(
    (a, b) => Number(b.main) - Number(a.main) || b.lastActivity - a.lastActivity,
  );
}

/** Every agent's conversations, as the sidebar reads them on connecting. */
export function withConversations(state: AppState, list: Conversation[]): Partial<AppState> {
  const byBot: Record<BotId, Conversation[]> = {};
  let replyAt = state.replyAt;
  for (const conversation of list) {
    byBot[conversation.botId] = [...(byBot[conversation.botId] ?? []), conversation];
    const at = conversation.lastReplyAt;
    // The main one's replies are the bot's own (`Bot.lastReplyAt`).
    if (!conversation.main && at !== null && at > (replyAt[conversation.id] ?? 0)) {
      replyAt = { ...replyAt, [conversation.id]: at };
    }
  }
  const conversations = Object.fromEntries(
    Object.entries(byBot).map(([botId, mine]) => [botId, ordered(mine)]),
  );
  return { conversations, replyAt };
}

/** A conversation that changed: in its agent's list, or out of it once archived. */
export function withConversation(state: AppState, conversation: Conversation): Partial<AppState> {
  const known = state.conversations[conversation.botId] ?? [];
  const others = known.filter((each) => each.id !== conversation.id);
  const list = conversation.archivedAt === null ? ordered([...others, conversation]) : others;
  return { conversations: { ...state.conversations, [conversation.botId]: list } };
}

/** The conversation open in the agent's chat: the one the owner left, while it is on the list. */
export function openConversationOf(state: AppState, botId: BotId): ConversationId {
  const wanted = state.openConversations[botId];
  const list = state.conversations[botId];
  if (wanted && (!list || list.some((each) => each.id === wanted))) {
    return wanted;
  }
  return mainConversationOf(botId);
}

/** What the daemon said of the open conversation; null until its list is read. */
export function conversationOf(state: AppState, botId: BotId): Conversation | null {
  const open = openConversationOf(state, botId);
  return state.conversations[botId]?.find((each) => each.id === open) ?? null;
}

/** The store's conversation actions. */
export function conversationActions(
  get: StoreApi<AppState>["getState"],
  set: StoreApi<AppState>["setState"],
  api: BotloftApi,
): Pick<AppState, "loadConversations" | "openConversation" | "newConversation"> {
  const open = (botId: BotId, conversationId: ConversationId) => {
    const next = { ...get().openConversations, [botId]: conversationId };
    saveOpen(next);
    set({ openConversations: next });
  };
  return {
    loadConversations: async (botId) => {
      const list = await api.call("conversations.list", { botId });
      set((state) => ({ conversations: { ...state.conversations, [botId]: ordered(list) } }));
    },
    openConversation: (botId, conversationId) => {
      open(botId, conversationId);
      if (get().selectedBotId !== botId) {
        get().selectBot(botId);
      }
    },
    newConversation: async (botId) => {
      const made = await api.call("conversations.create", { botId });
      set((state) => ({
        conversations: {
          ...state.conversations,
          [botId]: ordered([
            ...(state.conversations[botId] ?? []).filter((each) => each.id !== made.id),
            made,
          ]),
        },
      }));
      open(botId, made.id);
      return made;
    },
  };
}
