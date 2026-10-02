// The owner's choices that belong to this app and not to the daemon (spec
// 15.1): kept in localStorage, like the theme, and read at once where they
// apply.

import { useSyncExternalStore } from "react";

type Value = string | boolean;

export interface Pref<T extends Value> {
  get(): T;
  set(value: T): void;
  subscribe(listener: () => void): () => void;
  /** Back to the default, for tests. */
  reset(): void;
}

function pref<T extends Value>(key: string, fallback: T, allowed: readonly T[]): Pref<T> {
  const listeners = new Set<() => void>();
  const read = (): T => {
    try {
      const text = localStorage.getItem(key);
      return allowed.find((value) => String(value) === text) ?? fallback;
    } catch {
      return fallback;
    }
  };
  let current = read();
  const set = (value: T) => {
    current = value;
    try {
      localStorage.setItem(key, String(value));
    } catch {
      // Private storage off: the choice lasts for this window.
    }
    for (const listener of listeners) {
      listener();
    }
  };
  return {
    get: () => current,
    set,
    subscribe: (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    reset: () => set(fallback),
  };
}

const ON_OFF = [true, false] as const;

export const prefs = {
  /** What closing the window does: the bots keep working, or stop. */
  whenClosed: pref<"keep" | "stop">("botloft.whenClosed", "keep", ["keep", "stop"]),
  /** Enter sends a message; off, Ctrl+Enter does and Enter starts a line. */
  enterSends: pref<boolean>("botloft.enterSends", true, ON_OFF),
  /** The browser and the screens open by themselves when a bot starts on them. */
  followBot: pref<boolean>("botloft.followBot", true, ON_OFF),
  /** The crews and bots show on the left; off, only the selection does. */
  sidebar: pref<boolean>("botloft.sidebar", true, ON_OFF),
  /** No motion in the window, whatever Windows says. */
  lessMotion: pref<boolean>("botloft.lessMotion", false, ON_OFF),
  /** With the bots working, closing the window leaves Botloft near the clock. */
  tray: pref<boolean>("botloft.tray", true, ON_OFF),
  /** When Windows opens Botloft at sign-in, the window opens too. */
  openAtSignIn: pref<boolean>("botloft.openAtSignIn", false, ON_OFF),
  /** A Windows notification when a bot needs the owner. */
  notifyNeeds: pref<boolean>("botloft.notifyNeeds", true, ON_OFF),
  /** A Windows notification when a bot finishes what it was doing. */
  notifyDone: pref<boolean>("botloft.notifyDone", false, ON_OFF),
  /** The mark on the app's icon also shows while a bot is unread. */
  markReplies: pref<boolean>("botloft.markReplies", true, ON_OFF),
  /** A short sound with each of those. */
  sound: pref<boolean>("botloft.sound", true, ON_OFF),
};

export function usePref<T extends Value>(choice: Pref<T>): T {
  return useSyncExternalStore(choice.subscribe, choice.get);
}

export function resetPrefs(): void {
  for (const choice of Object.values(prefs)) {
    choice.reset();
  }
}
