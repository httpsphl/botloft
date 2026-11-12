// What the owner has seen of each bot (spec 15.1) and of each of its other
// conversations (spec 40.7), kept under the conversation's id: when they last looked at
// its chat, remembered across restarts of the app. A reply newer than that
// marks the bot unread in the conversation list; a routine run that failed
// after it marks the app's icon (`unseenFailures`).

import type { StoreApi } from "zustand/vanilla";
import type { BotloftApi } from "../lib/api";
import { mainConversationOf } from "../lib/conversations";
import type { BotId, Conversation, ConversationId, CrewId, Routine } from "../lib/protocol.gen";
import type { AppState } from "./app";
import { openConversationOf } from "./conversations";

const SEEN_KEY = "botloft.seen";
const SINCE_KEY = "botloft.seenSince";
const MARKED_KEY = "botloft.unread";

export function loadSeen(): Record<BotId, number> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(SEEN_KEY) ?? "{}");
    return typeof parsed === "object" && parsed !== null ? (parsed as Record<BotId, number>) : {};
  } catch {
    return {};
  }
}

export function saveSeen(seen: Record<BotId, number>): void {
  try {
    localStorage.setItem(SEEN_KEY, JSON.stringify(seen));
  } catch {
    // Storage may be off; the marks just show again after a restart.
  }
}

/**
 * When the app first kept unread marks: every reply before it counts as
 * seen, so the first start with them does not mark every bot unread.
 */
export function loadSeenSince(): number {
  try {
    const stored = Number(localStorage.getItem(SINCE_KEY));
    if (stored > 0) {
      return stored;
    }
    const now = Date.now();
    localStorage.setItem(SINCE_KEY, String(now));
    return now;
  } catch {
    return Date.now();
  }
}

/** The bots the owner marked unread, remembered across restarts. */
export function loadMarked(): Record<BotId, true> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(MARKED_KEY) ?? "[]");
    return Array.isArray(parsed)
      ? Object.fromEntries(parsed.filter((id) => typeof id === "string").map((id) => [id, true]))
      : {};
  } catch {
    return {};
  }
}

function saveMarked(marked: Record<BotId, true>): void {
  try {
    localStorage.setItem(MARKED_KEY, JSON.stringify(Object.keys(marked)));
  } catch {
    // Storage may be off; the marks last until the app closes.
  }
}

/** `replyAt` with a reply of `botId` at `at`, if it is newer. */
export function withReply(
  replyAt: Record<BotId, number>,
  botId: BotId,
  at: number | null,
): Record<BotId, number> {
  return at !== null && at > (replyAt[botId] ?? 0) ? { ...replyAt, [botId]: at } : replyAt;
}

/** The store's `markSeen` and `markUnread`. */
export function seenActions(
  get: StoreApi<AppState>["getState"],
  set: StoreApi<AppState>["setState"],
  api: BotloftApi,
): Pick<AppState, "markSeen" | "markSeenIn" | "markUnread"> {
  return {
    markSeenIn: (botId, conversationId) => {
      if (conversationId === mainConversationOf(botId)) {
        get().markSeen(botId);
        return;
      }
      const { seenAt, replyAt, markedUnread } = get();
      const next = {
        ...seenAt,
        [conversationId]: Math.max(Date.now(), replyAt[conversationId] ?? 0),
      };
      saveSeen(next);
      api.call("chat.read", { botId, conversationId }).catch(() => {});
      // Opening the agent's chat ends the mark the owner put on it.
      const { [botId]: wasMarked, ...marked } = markedUnread;
      if (wasMarked) {
        saveMarked(marked);
      }
      set(wasMarked ? { seenAt: next, markedUnread: marked } : { seenAt: next });
    },
    markSeen: (botId) => {
      const { seenAt, replyAt, markedUnread } = get();
      const next = { ...seenAt, [botId]: Math.max(Date.now(), replyAt[botId] ?? 0) };
      saveSeen(next);
      // The daemon keeps it too: this app's own storage has gone back to an
      // older state before, and the mark came back. A phone connected to
      // this computer also stops marking the chat new (spec 28.12).
      api.call("chat.read", { botId }).catch(() => {});
      const { [botId]: wasMarked, ...marked } = markedUnread;
      if (wasMarked) {
        saveMarked(marked);
      }
      set(wasMarked ? { seenAt: next, markedUnread: marked } : { seenAt: next });
    },
    markUnread: (botId) => {
      const { bots, markedUnread, selectedBotId, selectCrew } = get();
      const bot = bots[botId];
      if (!bot) {
        return;
      }
      const marked: Record<BotId, true> = { ...markedUnread, [botId]: true };
      saveMarked(marked);
      set({ markedUnread: marked });
      // An open chat is being read: marking it unread puts it away.
      if (selectedBotId === botId) {
        selectCrew(bot.crewId);
      }
    },
  };
}

/** The bot replied in its main conversation since the owner last looked, or they marked it unread. */
function mainUnread(state: AppState, botId: BotId): boolean {
  if (state.markedUnread[botId]) {
    return true;
  }
  const at = state.replyAt[botId];
  const read = state.bots[botId]?.readThrough ?? 0;
  return at !== undefined && at > Math.max(state.seenAt[botId] ?? 0, state.seenSince, read);
}

/** The bot replied in another conversation since the owner last read it. */
function otherUnread(state: AppState, conversation: Conversation): boolean {
  const at = state.replyAt[conversation.id] ?? conversation.lastReplyAt;
  const read = Math.max(
    state.seenAt[conversation.id] ?? 0,
    state.seenSince,
    conversation.readThrough ?? 0,
  );
  return at !== null && at !== undefined && at > read;
}

/** One conversation has a reply the owner has not seen; the open one never has. */
export function isConversationUnread(state: AppState, conversation: Conversation): boolean {
  const { botId, id } = conversation;
  if (botId === state.selectedBotId && openConversationOf(state, botId) === id) {
    return false;
  }
  return conversation.main ? mainUnread(state, botId) : otherUnread(state, conversation);
}

/**
 * The bot replied since the owner last looked at its chat, in any of its
 * conversations, or the owner marked it unread. The open chat is never unread.
 */
export function isUnread(state: AppState, botId: BotId): boolean {
  if (botId === state.selectedBotId || !state.bots[botId]) {
    return false;
  }
  if (mainUnread(state, botId)) {
    return true;
  }
  const others = state.conversations[botId] ?? [];
  return others.some((conversation) => !conversation.main && otherUnread(state, conversation));
}

/** The key a conversation's seen and reply marks are kept under. */
export function seenKey(botId: BotId, conversationId: ConversationId): string {
  return conversationId === mainConversationOf(botId) ? botId : conversationId;
}

/** How many bots of `crewId` are unread. */
export function unreadIn(state: AppState, crewId: CrewId): number {
  return Object.values(state.bots).filter((bot) => bot.crewId === crewId && isUnread(state, bot.id))
    .length;
}

/** Some bot is unread. */
export function anyUnread(state: AppState): boolean {
  return Object.keys(state.bots).some((botId) => isUnread(state, botId));
}

/** Routines whose last run failed after the owner last looked at their bot. */
export function unseenFailures(state: AppState): Routine[] {
  return Object.values(state.routines).filter((routine) => {
    const last = routine.lastRun;
    return (
      last?.status === "failed" &&
      routine.botId !== state.selectedBotId &&
      (last.finishedAt ?? 0) >
        Math.max(state.seenAt[routine.botId] ?? 0, state.bots[routine.botId]?.readThrough ?? 0)
    );
  });
}
