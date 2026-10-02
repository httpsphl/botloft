// The taskbar overlay (spec 15.2): a mark on the app's icon while something
// waits for the owner, so it shows even with the window in the background.

import { useEffect } from "react";
import { actionableDead } from "../features/messages/FailedDeliveries";
import type { AppState } from "../store/app";
import { useApp, useHost } from "../store/context";
import { unseenFailures } from "../store/seen";

/**
 * Bots waiting for an answer or a sign-in, messages that gave up, and
 * routine runs that failed since the owner last opened their bot.
 */
export function attentionCount(state: AppState): number {
  // Asked on every change to the store; the answer changes only with these.
  const inputs = [
    state.bots,
    state.deliveries,
    state.routines,
    state.seenAt,
    state.selectedBotId,
  ] as const;
  if (last?.inputs.every((input, index) => input === inputs[index])) {
    return last.count;
  }
  const bots = Object.values(state.bots).filter(
    (bot) => bot.state === "needs_approval" || bot.state === "auth_error",
  ).length;
  const count = bots + actionableDead(state).length + unseenFailures(state).length;
  last = { inputs, count };
  return count;
}

let last: { inputs: readonly unknown[]; count: number } | null = null;

export function useAttentionMark(): void {
  const host = useHost();
  const needed = useApp((state) => attentionCount(state) > 0);
  useEffect(() => {
    host.window.setAttention(needed).catch(() => {});
  }, [host, needed]);
}
