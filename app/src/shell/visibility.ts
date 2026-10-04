// Whether the owner can see the window: not minimized, not hidden near the
// clock (spec 15.1), not opened hidden at sign-in. While they cannot, the
// app stops animating (paused.css) and stops asking the daemon for what
// only the screen needs.
//
// Windows may not tell the page when the app hides its own window, so the
// app marks that itself; the window coming back to the front clears it.

import { useSyncExternalStore } from "react";

let hiddenByApp = false;
const listeners = new Set<() => void>();

export function windowHidden(): boolean {
  return hiddenByApp || document.visibilityState === "hidden";
}

function changed(): void {
  document.documentElement.toggleAttribute("data-hidden", windowHidden());
  for (const listener of listeners) {
    listener();
  }
}

/** The app hid its window, or showed it again. */
export function markHidden(hidden: boolean): void {
  if (hiddenByApp !== hidden) {
    hiddenByApp = hidden;
    changed();
  }
}

/** Calls `listener` when the window is hidden or shown; returns the unsubscribe. */
export function onVisibility(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/**
 * Whether the window has the focus: while the owner works in another app,
 * the mascots stand still (paused.css), so a Botloft left open in the
 * background costs next to nothing.
 */
function focusChanged(): void {
  document.documentElement.toggleAttribute("data-blurred", !document.hasFocus());
}

/** Starts following the window; once, before the app renders. */
export function watchVisibility(): void {
  document.addEventListener("visibilitychange", changed);
  window.addEventListener("focus", () => {
    markHidden(false);
    focusChanged();
  });
  window.addEventListener("blur", focusChanged);
  changed();
  focusChanged();
}

export function useWindowVisible(): boolean {
  return useSyncExternalStore(onVisibility, () => !windowHidden());
}
