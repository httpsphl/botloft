// How much an edit changed, on its tool line ("+12 −3"), and the diff the
// line opens to (spec 15.3). Each changed line carries its + or −, so the
// colors are never the only sign.

import { useT } from "../../i18n";
import type { EditLines } from "../../lib/protocol.gen";
import type { DiffLine } from "./lineDiff";

/** The counts the daemon took from the whole edit (spec 8.2); a zero side
 *  is left out, and an edit that changed nothing shows nothing. */
export function EditCounts({ lines }: { lines: EditLines }) {
  const t = useT();
  const { added, removed } = lines;
  if (added === 0 && removed === 0) {
    return null;
  }
  const said = [
    added > 0 && t.chat.tools.added(added),
    removed > 0 && t.chat.tools.removed(removed),
  ]
    .filter(Boolean)
    .join(", ");
  return (
    <span
      role="img"
      aria-label={said}
      title={said}
      className="flex shrink-0 gap-1 font-mono text-xs"
    >
      {added > 0 && <span className="text-ok">+{added}</span>}
      {removed > 0 && <span className="text-danger">−{removed}</span>}
    </span>
  );
}

const MARK: Record<DiffLine["kind"], string> = { same: " ", added: "+", removed: "−" };

const LINE: Record<DiffLine["kind"], string> = {
  same: "",
  added: "bg-ok/12",
  removed: "bg-danger/12",
};

const SIGN: Record<DiffLine["kind"], string> = {
  same: "text-muted",
  added: "text-ok",
  removed: "text-danger",
};

/** One block per replacement, each line with its mark. */
export function EditDiff({ diffs }: { diffs: DiffLine[][] }) {
  return (
    <div className="flex flex-col gap-1.5">
      {diffs.map((lines, n) => (
        <pre
          // biome-ignore lint/suspicious/noArrayIndexKey: a fixed list, with no ids
          key={n}
          className="max-h-64 overflow-auto rounded-lg border border-line bg-sunken py-1.5 font-mono text-xs leading-relaxed"
        >
          {lines.map((line, i) => (
            <span
              // biome-ignore lint/suspicious/noArrayIndexKey: lines of one fixed text
              key={i}
              data-kind={line.kind}
              className={`flex whitespace-pre-wrap break-all px-2.5 ${LINE[line.kind]}`}
            >
              <span
                aria-hidden={line.kind === "same"}
                className={`w-4 shrink-0 select-none ${SIGN[line.kind]}`}
              >
                {MARK[line.kind]}
              </span>
              <span className="min-w-0 flex-1">{line.text || " "}</span>
            </span>
          ))}
        </pre>
      ))}
    </div>
  );
}
