// The conversation list's state filter: all, needs you, working, unread. It
// lasts for this window only; a filter left on across restarts would look
// like missing bots.

import { useSyncExternalStore } from "react";
import type { Bot } from "../../lib/protocol.gen";
import type { AppState } from "../../store/app";
import { botsOf, crewList } from "../../store/app";
import { isUnread } from "../../store/seen";

export const STATE_FILTERS = ["all", "needs", "working", "unread"] as const;
export type StateFilter = (typeof STATE_FILTERS)[number];

let current: StateFilter = "all";
const listeners = new Set<() => void>();

export function setStateFilter(next: StateFilter): void {
  if (next === current) {
    return;
  }
  current = next;
  for (const listener of listeners) {
    listener();
  }
}

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => listeners.delete(listener);
};

export function useStateFilter(): StateFilter {
  return useSyncExternalStore(subscribe, () => current);
}

/** Back to all bots, for tests. */
export function resetStateFilter(): void {
  setStateFilter("all");
}

/** Whether a bot belongs under `filter`. Unread is passed in: it lives in the store. */
export function matchesFilter(filter: StateFilter, bot: Bot, unread: boolean): boolean {
  switch (filter) {
    case "needs":
      return bot.state === "needs_approval" || bot.state === "auth_error";
    case "working":
      return bot.state === "busy" || bot.state === "launching";
    case "unread":
      return unread;
    default:
      return true;
  }
}

/** The bots of a crew that pass `filter`. */
export function filteredBots(state: AppState, crewId: string, filter: StateFilter): Bot[] {
  const bots = botsOf(state, crewId);
  return filter === "all"
    ? bots
    : bots.filter((bot) => matchesFilter(filter, bot, isUnread(state, bot.id)));
}

/** How many bots each filter would show, across every crew. */
export function filterCounts(state: AppState): Record<StateFilter, number> {
  const counts: Record<StateFilter, number> = { all: 0, needs: 0, working: 0, unread: 0 };
  for (const crew of crewList(state)) {
    for (const bot of botsOf(state, crew.id)) {
      const unread = isUnread(state, bot.id);
      for (const filter of STATE_FILTERS) {
        if (matchesFilter(filter, bot, unread)) {
          counts[filter] += 1;
        }
      }
    }
  }
  return counts;
}
