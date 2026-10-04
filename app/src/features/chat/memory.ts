// A bot writing down what it learned (spec 15.3): an edit to its memory, the
// `CLAUDE.md` of its folder, or to the crew's, which every bot of the crew
// reads. The chat shows it as a line of its own, opening what changed.

import type { Bot, ChatItem, Crew, ToolItem } from "../../lib/protocol.gen";

export type MemoryOwner = "bot" | "crew";

/** Where a crew's memories live, as `memoryOf` compares them. */
export interface MemoryPlaces {
  bot: string[];
  crew: string[];
}

/** One stretch of the memory that changed: lines that left and came. */
export interface MemoryHunk {
  removed: string[];
  added: string[];
}

export type MemoryChange =
  | { kind: "edit"; hunks: MemoryHunk[] }
  /** The whole file written again: what it says now. */
  | { kind: "write"; text: string };

/** Windows paths compare without case and with either slash. */
function norm(path: string): string {
  return path.replaceAll("\\", "/").replace(/\/+/g, "/").replace(/\/$/, "").toLowerCase();
}

function parent(path: string): string {
  const cut = path.lastIndexOf("/");
  return cut > 0 ? path.slice(0, cut) : path;
}

const FILES = ["CLAUDE.md", ".claude/CLAUDE.md"];

function inside(folder: string): string[] {
  return FILES.map((file) => norm(`${folder}/${file}`));
}

export function memoryPlaces(bot: Pick<Bot, "workspace">, crew?: Pick<Crew, "workFolder">) {
  const workspace = norm(bot.workspace);
  return {
    bot: inside(workspace),
    // The crew folder holds every bot's folder (spec 5.1).
    crew: [norm(`${parent(workspace)}/CLAUDE.md`), ...(crew ? inside(crew.workFolder) : [])],
  } satisfies MemoryPlaces;
}

function isAbsolute(path: string): boolean {
  return /^([a-z]:)?[\\/]/i.test(path);
}

/** Whose memory a finished edit wrote; `null` for any other item. */
export function memoryOf(
  item: ChatItem,
  places: MemoryPlaces,
  workspace: string,
): MemoryOwner | null {
  const tool = item.body.kind === "tool" ? item.body : null;
  if (!tool?.file || tool.status !== "done") {
    return null;
  }
  const file = norm(isAbsolute(tool.file) ? tool.file : `${workspace}/${tool.file}`);
  if (places.bot.includes(file)) {
    return "bot";
  }
  return places.crew.includes(file) ? "crew" : null;
}

/** The lines both sides share at the start and end left out. */
function hunk(before: string, after: string): MemoryHunk | null {
  const removed = before.split("\n");
  const added = after.split("\n");
  while (removed.length && added.length && removed[0] === added[0]) {
    removed.shift();
    added.shift();
  }
  while (removed.length && added.length && removed.at(-1) === added.at(-1)) {
    removed.pop();
    added.pop();
  }
  return removed.length || added.length ? { removed, added } : null;
}

function text(value: unknown): string | null {
  return typeof value === "string" ? value : null;
}

/**
 * What the call changed, from its input; `null` when the input was cut
 * (spec 8.2) or is not an edit, and only the file is left to open.
 */
export function memoryChange(tool: ToolItem): MemoryChange | null {
  let input: Record<string, unknown>;
  try {
    input = JSON.parse(tool.input) as Record<string, unknown>;
  } catch {
    return null;
  }
  const content = text(input.content);
  if (content !== null) {
    return { kind: "write", text: content };
  }
  const edits = Array.isArray(input.edits) ? (input.edits as Record<string, unknown>[]) : [input];
  const hunks = edits.flatMap((edit) => {
    const before = text(edit?.old_string);
    const after = text(edit?.new_string);
    const change = before !== null && after !== null ? hunk(before, after) : null;
    return change ? [change] : [];
  });
  return hunks.length ? { kind: "edit", hunks } : null;
}
