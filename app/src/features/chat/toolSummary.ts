// A finished group of tool calls in one short line (spec 15.3): what kind
// of work it was, counted, in the order the bot did it, and how many
// calls failed.

import type { Messages } from "../../i18n/en";
import type { ToolItem } from "../../lib/protocol.gen";
import { toolKey } from "./toolNames";

type Words = Messages["chat"]["tools"]["group"];
type Kind = Exclude<keyof Words, "failed">;

const KINDS: Record<string, Kind> = {
  Bash: "command",
  PowerShell: "command",
  BashOutput: "command",
  KillShell: "command",
  KillBash: "command",
  Read: "read",
  LS: "read",
  Write: "edit",
  Edit: "edit",
  MultiEdit: "edit",
  NotebookEdit: "edit",
  Grep: "search",
  Glob: "search",
  WebFetch: "web",
  WebSearch: "web",
  send_message: "message",
};

function kindOf(name: string): Kind {
  const key = toolKey(name);
  return KINDS[key] ?? (key.startsWith("browser") ? "browser" : "other");
}

/** "Ran 4 commands, read 2 files (1 failed)". */
export function toolSummary(tools: Pick<ToolItem, "name" | "status">[], words: Words): string {
  const counts = new Map<Kind, number>();
  for (const tool of tools) {
    const kind = kindOf(tool.name);
    counts.set(kind, (counts.get(kind) ?? 0) + 1);
  }
  const text = [...counts].map(([kind, count]) => words[kind](count)).join(", ");
  const failed = tools.filter((tool) => tool.status === "failed").length;
  const line = text.charAt(0).toUpperCase() + text.slice(1);
  return failed > 0 ? `${line} (${words.failed(failed)})` : line;
}
