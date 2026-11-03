// What fills an agent's conversation (spec 33.3): Claude Code's last split,
// with the conversation made up to the total of now.

import type { ContextPart, ContextPartKind, ContextUsage } from "../../lib/protocol.gen";

/** One color per part, from the mascots' palette, the same in both themes. */
export const PART_COLORS: Record<ContextPartKind, string> = {
  messages: "var(--work)",
  instructions: "#8C9BB0",
  tools: "#FF7A59",
  connected_tools: "#3FBF6F",
  memory: "#A48BFF",
  skills: "#FFC857",
  agents: "#3DD9C1",
  other: "#B07A54",
  buffer: "var(--line-strong)",
};

/**
 * The parts as of now. Claude Code splits the conversation when it answers;
 * in between, the total grows with each model request, and what it grew by
 * is the conversation's. The room kept to compact sits outside the total.
 */
export function liveParts(context: ContextUsage): ContextPart[] {
  // An engine that gives only the total has no split to make up.
  if (context.parts.length === 0) {
    return [];
  }
  const fixed = context.parts
    .filter((part) => part.kind !== "messages" && part.kind !== "buffer")
    .reduce((sum, part) => sum + part.tokens, 0);
  const messages = Math.max(0, context.usedTokens - fixed);
  const rest = context.parts.filter((part) => part.kind !== "messages");
  return messages > 0 ? [{ kind: "messages", tokens: messages }, ...rest] : rest;
}

/** What is left of the window once the room kept to compact is set aside. */
export function freeTokens(context: ContextUsage, parts: ContextPart[]): number {
  const buffer = parts.find((part) => part.kind === "buffer")?.tokens ?? 0;
  return Math.max(0, context.windowTokens - context.usedTokens - buffer);
}

/** "CLAUDE.md" and the folder it is in, from an absolute path. */
export function fileAndFolder(path: string): { name: string; folder: string } {
  const pieces = path.split(/[\\/]+/).filter(Boolean);
  return { name: pieces.at(-1) ?? path, folder: pieces.at(-2) ?? "" };
}
