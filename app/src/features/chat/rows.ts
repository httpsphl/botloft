// How a chat reads on screen (spec 15.3): what others say stands alone,
// what the bot does in a turn runs together, a date line starts every day
// and a clock line marks a pause in the talk.

import type { ChatItem, Message, NoticeItem } from "../../lib/protocol.gen";
import { isSharedFiles } from "./shared";

export type Row =
  | { kind: "day"; key: string; at: number }
  /** The time the talk picked up again after a pause, on the same day. */
  | { kind: "time"; key: string; at: number }
  /** A message to the bot: from the owner, another bot or Botloft. */
  | { kind: "inbound"; key: string; item: ChatItem; message: Message }
  /** Replies, tool calls and approvals of one turn, and its end. */
  | { kind: "run"; key: string; items: ChatItem[] }
  | { kind: "notice"; key: string; item: ChatItem; notice: NoticeItem };

/**
 * Tools whose card says it all: the bot asking the owner (spec 23.2) or
 * for a key (spec 32.5), and the plan it shows before long work (spec 10.5).
 */
const CARD_TOOLS = new Set([
  "mcp__botloft__ask_owner",
  "mcp__botloft__ask_key",
  "mcp__botloft__show_plan",
]);

/** A call that worked and has its own card, which stands for it. */
function isAskLine(item: ChatItem): boolean {
  return (
    item.body.kind === "tool" && CARD_TOOLS.has(item.body.name) && item.body.status !== "failed"
  );
}

/** A pause this long before something new gets a clock line. */
export const PAUSE_MS = 30 * 60 * 1000;

function dayOf(ms: number): string {
  return new Date(ms).toDateString();
}

export function chatRows(items: ChatItem[]): Row[] {
  const rows: Row[] = [];
  let day: string | null = null;
  let run: ChatItem[] | null = null;
  let last = 0;
  for (const item of items) {
    if (isAskLine(item)) {
      continue;
    }
    const today = dayOf(item.createdAt);
    const body = item.body;
    // What the bot does inside one turn never gets cut by a clock line.
    const opens = !run || body.kind === "inbound" || body.kind === "notice";
    if (today !== day) {
      day = today;
      run = null;
      rows.push({ kind: "day", key: `day-${item.id}`, at: item.createdAt });
    } else if (opens && item.createdAt - last >= PAUSE_MS) {
      run = null;
      rows.push({ kind: "time", key: `time-${item.id}`, at: item.createdAt });
    }
    last = item.createdAt;
    if (body.kind === "inbound") {
      run = null;
      rows.push({ kind: "inbound", key: item.id, item, message: body.message });
    } else if (body.kind === "notice") {
      run = null;
      rows.push({ kind: "notice", key: item.id, item, notice: body });
    } else {
      if (!run) {
        run = [];
        rows.push({ kind: "run", key: item.id, items: run });
      }
      run.push(item);
      if (body.kind === "turn") {
        run = null;
      }
    }
  }
  return rows;
}

/** Whether the row shows the chat item `id`. */
export function rowHas(row: Row, id: string): boolean {
  switch (row.kind) {
    case "day":
    case "time":
      return false;
    case "run":
      return row.items.some((item) => item.id === id);
    default:
      return row.item.id === id;
  }
}

/** A card still waiting for the owner: a request, a question, a key. */
export function isWaiting(item: ChatItem): boolean {
  switch (item.body.kind) {
    case "approval":
      return item.body.status === "pending";
    case "question":
      return item.body.question.status === "open";
    case "key_request":
      return item.body.status === "open";
    default:
      return false;
  }
}

/** Tool calls that run together as lines; shared files are cards of their own. */
function isLine(item: ChatItem | undefined): boolean {
  return item?.body.kind === "tool" && !isSharedFiles(item);
}

/**
 * Splits a run into replies, approvals, shared files and turns alone, and
 * tool calls together. `alone` keeps other calls out of the group, such as
 * an edit to the bot's memory.
 */
export function runParts(
  items: ChatItem[],
  alone: (item: ChatItem) => boolean = () => false,
): ChatItem[][] {
  const parts: ChatItem[][] = [];
  const line = (item: ChatItem | undefined) => isLine(item) && !(item && alone(item));
  for (const item of items) {
    const last = parts.at(-1);
    if (last && line(item) && line(last[0])) {
      last.push(item);
    } else {
      parts.push([item]);
    }
  }
  return parts;
}

/**
 * Splits the reply being written where its last finished block ends: the
 * part before no longer changes, so only the rest is parsed again as text
 * arrives. A blank line inside a code fence does not end a block.
 */
export function splitDraft(text: string): [string, string] {
  let fence: string | null = null;
  let blank = false;
  let cut = 0;
  let offset = 0;
  for (const line of text.split("\n")) {
    if (fence === null && blank && /^\S/.test(line)) {
      cut = offset;
    }
    const marker = /^ {0,3}(`{3,}|~{3,})/.exec(line)?.[1];
    if (marker && fence === null) {
      fence = marker;
    } else if (
      marker &&
      fence !== null &&
      marker[0] === fence[0] &&
      marker.length >= fence.length
    ) {
      fence = null;
    }
    blank = line.trim() === "";
    offset += line.length + 1;
  }
  return [text.slice(0, cut), text.slice(cut)];
}
