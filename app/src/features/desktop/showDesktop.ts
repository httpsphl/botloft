// How a desktop tool line in the chat opens the bot's desktop panel (spec
// 24.9). Absent where there is no panel, so no button shows.

import { createContext } from "react";

export const ShowDesktop = createContext<(() => void) | null>(null);

/** `mcp__botloft__desktop_look` and the other desktop tools. */
export function isDesktopTool(name: string): boolean {
  return name.startsWith("mcp__botloft__desktop_");
}
