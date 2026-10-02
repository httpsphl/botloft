// What the owner has seen of each bot (spec 15.1): when they last looked at
// its chat, remembered across restarts of the app. A reply newer than that
// marks the bot unread in the conversation list; a routine run that failed
// after it marks the app's icon (`unseenFailures`).

import type { BotId, Routine } from "../lib/protocol.gen";
import type { AppState } from "./app";

const SEEN_KEY = "botloft.seen";
const SINCE_KEY = "botloft.seenSince";

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

/** The bot replied since the owner last looked at its chat. */
export function hasUnreadReply(state: AppState, botId: BotId): boolean {
  if (botId === state.selectedBotId) {
    return false;
  }
  const at = state.replyAt[botId];
  return at !== undefined && at > Math.max(state.seenAt[botId] ?? 0, state.seenSince);
}

/** Some bot replied since the owner last looked at its chat. */
export function anyUnreadReply(state: AppState): boolean {
  return Object.keys(state.replyAt).some((botId) => hasUnreadReply(state, botId));
}

/** Routines whose last run failed after the owner last looked at their bot. */
export function unseenFailures(state: AppState): Routine[] {
  return Object.values(state.routines).filter((routine) => {
    const last = routine.lastRun;
    return (
      last?.status === "failed" &&
      routine.botId !== state.selectedBotId &&
      (last.finishedAt ?? 0) > (state.seenAt[routine.botId] ?? 0)
    );
  });
}
