// Helpers for motion (spec 15.3): telling what is new since the owner
// started looking, so only that animates in.

import { createContext, useContext } from "react";

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
