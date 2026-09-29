// The taskbar overlay (spec 15.2): a mark on the app's icon while something
// waits for the owner, so it shows even with the window in the background.

import { useEffect } from "react";
import { actionableDead } from "../features/messages/FailedDeliveries";
import { type AppState, unseenFailures } from "../store/app";
import { useApp, useHost } from "../store/context";

/**
 * Bots waiting for an answer or a sign-in, messages that gave up, and
 * routine runs that failed since the owner last opened their bot.
 */
export function attentionCount(state: AppState): number {
  const bots = Object.values(state.bots).filter(
    (bot) => bot.state === "needs_approval" || bot.state === "auth_error",
  ).length;
  return bots + actionableDead(state).length + unseenFailures(state).length;
}

export function useAttentionMark(): void {
  const host = useHost();
  const needed = useApp((state) => attentionCount(state) > 0);
  useEffect(() => {
    host.window.setAttention(needed).catch(() => {});
  }, [host, needed]);
}
