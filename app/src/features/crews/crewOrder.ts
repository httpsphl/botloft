// The order of crews in the sidebar and on the crews page (spec 15.1): the
// owner drags them, or moves them from the menu. A look of this computer,
// like which crews are folded, so it lives in localStorage; a crew not in the
// saved order (a new one) goes last, in the order it was made.

import { useSyncExternalStore } from "react";
import type { Crew, CrewId } from "../../lib/protocol.gen";

const KEY = "botloft.crewOrder";

const listeners = new Set<() => void>();
let current = load();

function load(): readonly CrewId[] {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    return Array.isArray(parsed) ? parsed.filter((id) => typeof id === "string") : [];
  } catch {
    return [];
  }
}

/** Saves the order of every crew shown, first to last. */
export function setCrewOrder(ids: readonly CrewId[]): void {
  current = [...ids];
  try {
    localStorage.setItem(KEY, JSON.stringify(current));
  } catch {
    // Storage off: the order lasts for this window.
  }
  for (const listener of listeners) {
    listener();
  }
}

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => listeners.delete(listener);
};

export function useCrewOrder(): readonly CrewId[] {
  return useSyncExternalStore(subscribe, () => current);
}

/** `crews` (in creation order) arranged by the saved order. */
export function orderedCrews(crews: Crew[], order: readonly CrewId[]): Crew[] {
  const rank = new Map(order.map((id, index) => [id, index]));
  const known = crews.filter((crew) => rank.has(crew.id));
  known.sort((a, b) => (rank.get(a.id) ?? 0) - (rank.get(b.id) ?? 0));
  return [...known, ...crews.filter((crew) => !rank.has(crew.id))];
}

/** `ids` with `id` taken out and put next to `target` (before it, or after). */
export function moveBeside(
  ids: readonly CrewId[],
  id: CrewId,
  target: CrewId,
  after: boolean,
): CrewId[] {
  if (id === target) {
    return [...ids];
  }
  const rest = ids.filter((other) => other !== id);
  const at = rest.indexOf(target);
  rest.splice(after ? at + 1 : at, 0, id);
  return rest;
}

/** One place up or down; stays put at the ends. */
export function moveBy(ids: readonly CrewId[], id: CrewId, delta: -1 | 1): CrewId[] {
  const from = ids.indexOf(id);
  const to = from + delta;
  const next = [...ids];
  if (from < 0 || to < 0 || to >= next.length) {
    return next;
  }
  next.splice(from, 1);
  next.splice(to, 0, id);
  return next;
}

/** Back to creation order, for tests. */
export function resetCrewOrder(): void {
  setCrewOrder([]);
}
