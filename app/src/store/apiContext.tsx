// React access to the daemon API alone, without the app store: a mascot
// reads it here, so the pages that show mascots outside the app (Setup, the
// public site) do not carry the whole store with them.

import { createContext, type ReactNode, useContext } from "react";
import type { BotloftApi } from "../lib/api";

export const ApiContext = createContext<BotloftApi | null>(null);

/** Provides `api` alone, for a page with no store of its own: the island
 *  (spec 31.3) reads with the home's hooks. */
export function ApiProvider({ api, children }: { api: BotloftApi; children: ReactNode }) {
  return <ApiContext.Provider value={api}>{children}</ApiContext.Provider>;
}

/** The API where there is one: a mascot also shows outside the app (Setup). */
export function useOptionalApi(): BotloftApi | null {
  return useContext(ApiContext);
}
