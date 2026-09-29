// How a tool line in the chat opens its screen in the design area (spec
// 22.5). Absent where there is no panel, so no button shows.

import { createContext } from "react";

export const ShowScreen = createContext<((path: string) => void) | null>(null);

/** An HTML file: a screen. */
export function isScreenFile(path: string): boolean {
  return /\.html?$/i.test(path);
}
