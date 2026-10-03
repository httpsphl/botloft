// How a chat reads on screen (spec 15.3): what others say stands alone,
// what the bot does in a turn runs together under its avatar, and a date
// line starts every day.

import type { ChatItem, Message, NoticeItem } from "../../lib/protocol.gen";
import { isSharedFiles } from "./shared";

export type Row =
  | { kind: "day"; key: string; at: number }
  /** A message to the bot: from the owner, another bot or Botloft. */
  | { kind: "inbound"; key: string; item: ChatItem; message: Message }
  /** Replies, tool calls and approvals of one turn, and its end. */
  | { kind: "run"; key: string; items: ChatItem[] }
  | { kind: "notice"; key: string; item: ChatItem; notice: NoticeItem };

/** The bot asking the owner (spec 23.2): its question card says it all. */
const ASK_TOOL = "mcp__botloft__ask_owner";

/** A call to ask the owner that worked; the card stands for it. */
function isAskLine(item: ChatItem): boolean {
  return item.body.kind === "tool" && item.body.name === ASK_TOOL && item.body.status !== "failed";
}

function dayOf(ms: number): string {
  return new Date(ms).toDateString();
}

export function chatRows(items: ChatItem[]): Row[] {
  const rows: Row[] = [];
  let day: string | null = null;
  let run: ChatItem[] | null = null;
  for (const item of items) {
    if (isAskLine(item)) {
      continue;
    }
    const today = dayOf(item.createdAt);
    if (today !== day) {
      day = today;
      run = null;
      rows.push({ kind: "day", key: `day-${item.id}`, at: item.createdAt });
    }
    const body = item.body;
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

/** Tool calls that run together as lines; shared files are cards of their own. */
function isLine(item: ChatItem | undefined): boolean {
  return item?.body.kind === "tool" && !isSharedFiles(item);
}

/**
 * Splits a run into replies, approvals, shared files and turns alone, and
 * tool calls together.
 */
export function runParts(items: ChatItem[]): ChatItem[][] {
  const parts: ChatItem[][] = [];
  for (const item of items) {
    const last = parts.at(-1);
    if (last && isLine(item) && isLine(last[0])) {
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
