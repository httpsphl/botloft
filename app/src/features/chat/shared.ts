// What a `share_file` call shared (spec 10): the daemon describes each file
// in the tool's answer, as the files panel lists it.

import type { BotFile, ChatItem, ToolItem } from "../../lib/protocol.gen";
import { toolKey } from "./toolNames";

function isBotFile(value: unknown): value is BotFile {
  const file = value as Partial<BotFile> | null;
  return (
    typeof file?.path === "string" &&
    typeof file.name === "string" &&
    typeof file.mediaType === "string" &&
    typeof file.size === "number"
  );
}

/** The files the daemon described in the tool's answer. */
export function sharedFiles(tool: ToolItem): BotFile[] {
  try {
    const answer: unknown = JSON.parse(tool.output ?? "");
    const shown = (answer as { shown?: unknown } | null)?.shown;
    return Array.isArray(shown) ? shown.filter(isBotFile) : [];
  } catch {
    return [];
  }
}

/**
 * A `share_file` call that worked shows as file cards, not as a tool line.
 * One whose answer cannot be read stays a tool line, where it can be opened.
 */
export function isSharedFiles(item: ChatItem): boolean {
  return (
    item.body.kind === "tool" &&
    toolKey(item.body.name) === "share_file" &&
    item.body.status === "done" &&
    sharedFiles(item.body).length > 0
  );
}
