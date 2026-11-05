// What the bot made in a turn without sharing it (spec 15.1): the pictures,
// sounds and videos among its files whose last change falls inside the
// turn. A command or script never says what it wrote, so the files list is
// where they show up; the chat puts them under the turn they came from.

import { createContext } from "react";
import type { BotFile, ChatItem } from "../../lib/protocol.gen";
import { fileKind } from "../files/kinds";
import { isSharedFiles, sharedFiles } from "./shared";

/** Cards shown under a turn; the rest is a link to the files panel. */
export const TURN_MEDIA_MAX = 6;
/** Room for clocks that round, on both sides of the turn. */
const SLACK_MS = 2000;
const MEDIA = new Set(["image", "audio", "video"]);

/**
 * The bot's files as the panel lists them, and how to open the panel. Absent
 * where there is no panel: no turn then shows what it made.
 */
export const TurnFiles = createContext<{ files: BotFile[]; open(): void } | null>(null);

/** A path as Windows compares it: case and slashes do not matter. */
function comparable(path: string): string {
  return path.replaceAll("/", "\\").toLowerCase();
}

/**
 * The media files changed during a turn that ended at `at` and lasted
 * `durationMs`, oldest first, except the ones it shared.
 */
export function madeInTurn(
  files: BotFile[],
  turn: { at: number; durationMs: number },
  shared: string[],
): BotFile[] {
  const start = turn.at - turn.durationMs - SLACK_MS;
  const end = turn.at + SLACK_MS;
  const skip = new Set(shared.map(comparable));
  return files
    .filter(
      (file) =>
        file.modifiedAt >= start &&
        file.modifiedAt <= end &&
        MEDIA.has(fileKind(file.mediaType, file.name)) &&
        !skip.has(comparable(file.path)),
    )
    .sort((a, b) => a.modifiedAt - b.modifiedAt || a.name.localeCompare(b.name));
}

/** The paths a run shared with `share_file`: they have their own cards. */
export function sharedIn(items: ChatItem[]): string[] {
  return items.flatMap((item) =>
    item.body.kind === "tool" && isSharedFiles(item)
      ? sharedFiles(item.body).map((file) => file.path)
      : [],
  );
}
