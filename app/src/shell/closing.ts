// What closing the window does (spec 15.1): Botloft keeps the bots working
// in the background, or, if the owner chose so in Settings, stops them
// until it opens again. The choice is this app's, kept in localStorage
// like the theme.

import { useEffect, useSyncExternalStore } from "react";
import type { Host } from "../lib/host";

export type WhenClosed = "keep" | "stop";

const KEY = "botloft.whenClosed";
const listeners = new Set<() => void>();

function stored(): WhenClosed {
  try {
    return localStorage.getItem(KEY) === "stop" ? "stop" : "keep";
  } catch {
    return "keep";
  }
}

let choice: WhenClosed = stored();

/** The owner's choice, for code outside React. */
export function whenClosed(): WhenClosed {
  return choice;
}

export function setWhenClosed(next: WhenClosed): void {
  choice = next;
  try {
    localStorage.setItem(KEY, next);
  } catch {
    // Private storage off: the choice lasts for this window.
  }
  for (const listener of listeners) {
    listener();
  }
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function useWhenClosed(): WhenClosed {
  return useSyncExternalStore(subscribe, () => choice);
}

/**
 * Stops the bots as the window closes, when the owner chose so. The window
 * hides first, since stopping takes a moment; if it fails, it closes all
 * the same.
 */
export function useCloseBehavior(host: Host): void {
  useEffect(() => {
    let off: (() => void) | undefined;
    let alive = true;
    host.window
      .onCloseRequested(async () => {
        if (choice !== "stop") {
          return;
        }
        await host.window.hide().catch(() => {});
        await host.stopDaemon().catch(() => {});
      })
      .then(
        (unsubscribe) => {
          if (alive) {
            off = unsubscribe;
          } else {
            unsubscribe();
          }
        },
        () => {},
      );
    return () => {
      alive = false;
      off?.();
    };
  }, [host]);
}
