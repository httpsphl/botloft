// Crews folded in the sidebar (spec 15.1): only the name shows, so many
// crews stay readable. Remembered across restarts in localStorage.

import { useSyncExternalStore } from "react";
import type { CrewId } from "../../lib/protocol.gen";

const KEY = "botloft.collapsedCrews";

const listeners = new Set<() => void>();
let current = load();

function load(): ReadonlySet<CrewId> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    return new Set(Array.isArray(parsed) ? parsed.filter((id) => typeof id === "string") : []);
  } catch {
    return new Set();
  }
}

function save(next: ReadonlySet<CrewId>): void {
  current = next;
  try {
    localStorage.setItem(KEY, JSON.stringify([...next]));
  } catch {
    // Storage off: the choice lasts for this window.
  }
  for (const listener of listeners) {
    listener();
  }
}

export function setCollapsed(crewId: CrewId, collapsed: boolean): void {
  if (current.has(crewId) === collapsed) {
    return;
  }
  const next = new Set(current);
  if (collapsed) {
    next.add(crewId);
  } else {
    next.delete(crewId);
  }
  save(next);
}

/** Folds every crew, or opens every one with an empty list. */
export function setAllCollapsed(crewIds: CrewId[]): void {
  save(new Set(crewIds));
}

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => listeners.delete(listener);
};

export function useCollapsed(crewId: CrewId): boolean {
  return useSyncExternalStore(subscribe, () => current.has(crewId));
}

export function useCollapsedSet(): ReadonlySet<CrewId> {
  return useSyncExternalStore(subscribe, () => current);
}

/** Back to every crew open, for tests. */
export function resetCollapsed(): void {
  save(new Set());
}
