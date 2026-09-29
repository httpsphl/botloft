// Helpers for motion (spec 15.3): switching views with a short transition,
// and telling what is new since the owner started looking, so only that
// animates in.

import { createContext, useContext } from "react";
import { flushSync } from "react-dom";

/** Whether Windows asked for less motion ("Animation effects" off). */
export function reducedMotion(): boolean {
  return (
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches
  );
}

type WithTransitions = Document & { startViewTransition?(update: () => void): unknown };

/**
 * Applies a change of view (another bot or crew) as a short transition of
 * the main pane, where the webview can; otherwise at once.
 */
export function viewTransition(update: () => void): void {
  const doc = document as WithTransitions;
  if (typeof doc.startViewTransition !== "function" || reducedMotion()) {
    update();
    return;
  }
  doc.startViewTransition(() => flushSync(update));
}

/** When the app opened: a bot created after it is new to the owner. */
export const APP_OPENED = Date.now();

/**
 * When the owner opened what they look at (a chat). Items created before
 * it were already there and stay still; later ones animate in.
 */
export const SeenSince = createContext<number>(Number.POSITIVE_INFINITY);

/** The class that animates an item in, if it is new. */
export function useArrival(createdAt: number): string {
  return createdAt > useContext(SeenSince) ? "animate-rise" : "";
}
