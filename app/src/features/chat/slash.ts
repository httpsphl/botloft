// What the menu of "/" looks for (spec 38.11): the name after the slash
// while the message is only that, and the agent's commands that match it.

import type { SlashCommand } from "../../lib/protocol.gen";

/** The part of a name typed after "/", or null when the message is not
 * only "/" and a name. */
export function slashQuery(text: string): string | null {
  const match = /^\/(\S*)$/.exec(text);
  return match ? (match[1] ?? "") : null;
}

/** The commands whose name has `query`, the ones that start with it first. */
export function matching(commands: SlashCommand[], query: string): SlashCommand[] {
  const wanted = query.toLowerCase();
  const starts: SlashCommand[] = [];
  const has: SlashCommand[] = [];
  for (const command of commands) {
    const name = command.name.toLowerCase();
    if (name.startsWith(wanted)) {
      starts.push(command);
    } else if (name.includes(wanted)) {
      has.push(command);
    }
  }
  return [...starts, ...has];
}
