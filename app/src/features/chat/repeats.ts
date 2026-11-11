// The same call made again and again in one turn (spec 15.3): the same
// tool, about the same thing, with the same input. A bot that keeps making
// it may be going in circles, so its group of calls says how many times,
// from the fifth on, while there is still time to stop it.

import type { ChatItem, ToolItem } from "../../lib/protocol.gen";
import { toolKey } from "./toolNames";

/** From how many of the same call the group shows it. */
export const REPEAT_FROM = 5;

/**
 * Tools made to be called again until something changes: a look at the
 * page or the screen, the output of a command left running.
 */
const POLLS = new Set([
  "BashOutput",
  "TaskOutput",
  "browser_look",
  "browser_screenshot",
  "browser_get_dom",
  "desktop_look",
  "desktop_screenshot",
  "desktop_windows",
]);

type Call = Pick<ToolItem, "name" | "summary" | "input">;

function sameness(tool: Call): string | null {
  return POLLS.has(toolKey(tool.name)) ? null : `${tool.name}\n${tool.summary}\n${tool.input}`;
}

/** For each call of a turn, how many times the same call had been made when it ran, itself included. */
export function repeatCounts(items: ChatItem[]): Map<string, number> {
  const made = new Map<string, number>();
  const counts = new Map<string, number>();
  for (const item of items) {
    if (item.body.kind !== "tool") {
      continue;
    }
    const same = sameness(item.body);
    if (same === null) {
      continue;
    }
    const count = (made.get(same) ?? 0) + 1;
    made.set(same, count);
    counts.set(item.id, count);
  }
  return counts;
}

export interface Repeat {
  count: number;
  tool: ToolItem;
}

/** The call of `group` made the most times so far, once that reaches {@link REPEAT_FROM}. */
export function groupRepeat(group: ChatItem[], counts: Map<string, number>): Repeat | null {
  let best: Repeat | null = null;
  for (const item of group) {
    const count = counts.get(item.id) ?? 0;
    if (item.body.kind === "tool" && count >= REPEAT_FROM && count > (best?.count ?? 0)) {
      best = { count, tool: item.body };
    }
  }
  return best;
}
