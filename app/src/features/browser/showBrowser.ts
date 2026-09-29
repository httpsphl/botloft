// How a browser tool line in the chat opens the browser panel (spec 21.8),
// and how a request for a hand opens it in the owner's hands (spec 21.10).
// Absent where there is no panel, so no button shows.

import { createContext } from "react";

export const ShowBrowser = createContext<((options?: { take?: boolean }) => void) | null>(null);

/** `mcp__botloft__browser_open` and the other browser tools. */
export function isBrowserTool(name: string): boolean {
  return name.startsWith("mcp__botloft__browser_");
}
