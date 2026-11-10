// Whether a bot's reply ends with a question, so that a 👍 or 👎 on it
// answers that question at once (spec 8.9). The same rule as the daemon's
// `crates/botloft-core/src/asks.rs`.

/** The reactions that answer a question. */
export const ANSWERS = ["👍", "👎"] as const;

/** What can close a line after its question mark. */
const CLOSERS = /[\s*_`"'”’»)\]]+$/u;

/** Whether the last line of `text` asks something. */
export function asks(text: string): boolean {
  const lines = text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "");
  const last = lines.at(-1);
  if (last === undefined) {
    return false;
  }
  const end = last.replace(CLOSERS, "");
  return end.endsWith("?") || end.endsWith("？");
}

/** Whether `emoji` on a reply that `asked` is sent at once as the answer. */
export function answers(emoji: string, asked: boolean): boolean {
  return asked && (ANSWERS as readonly string[]).includes(emoji);
}
