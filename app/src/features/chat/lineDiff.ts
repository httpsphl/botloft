// What a file edit changes, line by line (spec 15.3): the diff a `Write`,
// `Edit` or `MultiEdit` line opens to, from the call's input when it came
// whole. The daemon counts the same lines the same way (spec 8.2), so the
// diff and the "+N −M" on the line agree.

export interface DiffLine {
  kind: "same" | "added" | "removed";
  text: string;
}

/** Most pairs of lines compared; past it, every line left after the common
 *  start and end counts as changed, as in the daemon. */
const MAX_PAIRS = 4_000_000;

/** Lines as the daemon reads them: a final line break ends the last line,
 *  and a carriage return before a break is not part of the line. */
export function splitLines(text: string): string[] {
  if (text === "") {
    return [];
  }
  const lines = text.split("\n");
  if (lines[lines.length - 1] === "") {
    lines.pop();
  }
  return lines.map((line) => (line.endsWith("\r") ? line.slice(0, -1) : line));
}

/** The lines of `old` and `next` in order, each kept, added or removed:
 *  the longest run they share stays, the rest changes, removed first. */
export function diffLines(old: string, next: string): DiffLine[] {
  const a = splitLines(old);
  const b = splitLines(next);
  let start = 0;
  while (start < a.length && start < b.length && a[start] === b[start]) {
    start += 1;
  }
  let end = 0;
  while (
    end < a.length - start &&
    end < b.length - start &&
    a[a.length - 1 - end] === b[b.length - 1 - end]
  ) {
    end += 1;
  }
  const same = (text: string): DiffLine => ({ kind: "same", text });
  const middleA = a.slice(start, a.length - end);
  const middleB = b.slice(start, b.length - end);
  return [
    ...a.slice(0, start).map(same),
    ...middle(middleA, middleB),
    ...a.slice(a.length - end).map(same),
  ];
}

function middle(a: string[], b: string[]): DiffLine[] {
  const removed = (text: string): DiffLine => ({ kind: "removed", text });
  const added = (text: string): DiffLine => ({ kind: "added", text });
  if (a.length * b.length > MAX_PAIRS) {
    return [...a.map(removed), ...b.map(added)];
  }
  // run(i, j): the longest run `a` from i and `b` from j share.
  const width = b.length + 1;
  const table = new Uint32Array((a.length + 1) * width);
  const run = (i: number, j: number) => table[i * width + j] ?? 0;
  for (let i = a.length - 1; i >= 0; i -= 1) {
    for (let j = b.length - 1; j >= 0; j -= 1) {
      table[i * width + j] =
        a[i] === b[j] ? run(i + 1, j + 1) + 1 : Math.max(run(i + 1, j), run(i, j + 1));
    }
  }
  const lines: DiffLine[] = [];
  let i = 0;
  let j = 0;
  while (i < a.length || j < b.length) {
    const left = a[i];
    const right = b[j];
    if (left !== undefined && left === right) {
      lines.push({ kind: "same", text: left });
      i += 1;
      j += 1;
    } else if (left !== undefined && (right === undefined || run(i + 1, j) >= run(i, j + 1))) {
      lines.push(removed(left));
      i += 1;
    } else {
      lines.push(added(right ?? ""));
      j += 1;
    }
  }
  return lines;
}

function text(value: unknown, name: string): string | null {
  const field =
    value && typeof value === "object" ? (value as Record<string, unknown>)[name] : null;
  return typeof field === "string" ? field : null;
}

/** One diff per replacement of an edit call, from its input; `null` for
 *  other tools, an input cut to size and one without the texts (Codex and
 *  Antigravity send only the path). */
export function editDiffs(name: string, input: string): DiffLine[][] | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(input);
  } catch {
    return null;
  }
  const replacement = (edit: unknown) => {
    const old = text(edit, "old_string");
    const next = text(edit, "new_string");
    return old === null || next === null ? null : diffLines(old, next);
  };
  switch (name) {
    case "Write": {
      const content = text(parsed, "content");
      return content === null ? null : [diffLines("", content)];
    }
    case "Edit": {
      const diff = replacement(parsed);
      return diff && [diff];
    }
    case "MultiEdit": {
      const edits = (parsed as { edits?: unknown } | null)?.edits;
      if (!Array.isArray(edits) || edits.length === 0) {
        return null;
      }
      const diffs = edits.map(replacement);
      return diffs.every((diff) => diff !== null) ? (diffs as DiffLine[][]) : null;
    }
    default:
      return null;
  }
}

/** Whether `name` changes a file the way `editDiffs` reads. */
export function isEditTool(name: string): boolean {
  return name === "Write" || name === "Edit" || name === "MultiEdit";
}
