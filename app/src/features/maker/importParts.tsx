// What both ways of bringing a history to the Maker share (spec 36.7): the
// summary being made, read before it goes, then sent or forgotten, and the
// small parts of their dialogs.

import { FileClock } from "lucide-react";
import { type ReactNode, useState } from "react";
import { currentLocale, useT } from "../../i18n";
import type { ImportDraft, ImportsCreateParams } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt, notify } from "../../ui/toast";

type Kind = "key" | "email" | "number" | "card";
const KINDS: Kind[] = ["key", "email", "number", "card"];

/** "1 Oct – 3 Nov" in the owner's language. */
export function period(from: number | null, to: number | null): string {
  if (from === null || to === null) return "";
  const format = new Intl.DateTimeFormat(currentLocale(), { day: "numeric", month: "short" });
  return `${format.format(from)} – ${format.format(to)}`;
}

export function Check({
  label,
  checked,
  onChange,
  hint,
}: {
  label: string;
  checked: boolean;
  onChange(on: boolean): void;
  hint?: string;
}) {
  return (
    <label className="flex cursor-pointer items-start gap-2.5 rounded-lg px-2 py-1.5 hover:bg-sunken">
      <input
        type="checkbox"
        className="mt-1 accent-[var(--accent)]"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="min-w-0">
        <span className="block truncate font-medium text-sm" data-selectable>
          {label}
        </span>
        {hint && <span className="block text-muted text-xs">{hint}</span>}
      </span>
    </label>
  );
}

/** A summary being made, then sent to the Maker or forgotten. */
export function useImportDraft(onClose: () => void) {
  const h = useT().maker.history;
  const api = useApi();
  const [draft, setDraft] = useState<ImportDraft | null>(null);
  const [busy, setBusy] = useState(false);

  /** Makes the summary again, forgetting the one before. */
  const make = async (params: ImportsCreateParams) => {
    setBusy(true);
    const ok = await attempt(h.makeFailed, async () => {
      if (draft) await api.call("imports.forget", { importId: draft.importId });
      setDraft(null);
      setDraft(await api.call("imports.create", params));
    });
    setBusy(false);
    return ok;
  };

  const forget = async () => {
    if (draft) {
      await attempt(h.makeFailed, () => api.call("imports.forget", { importId: draft.importId }));
    }
    onClose();
  };

  const send = async () => {
    if (!draft) return;
    setBusy(true);
    const ok = await attempt(h.sendFailed, () =>
      api.call("imports.send", { importId: draft.importId }),
    );
    setBusy(false);
    if (ok) {
      notify(h.sent);
      onClose();
    }
  };

  const footer = draft && (
    <>
      <Button variant="ghost" onClick={forget} disabled={busy}>
        {h.forget}
      </Button>
      <Button variant="primary" onClick={send} disabled={busy}>
        {h.send}
      </Button>
    </>
  );

  return { draft, busy, setBusy, make, forget, footer };
}

/** The summary to read before it goes: its cost, what was swapped, all of it. */
export function DraftReview({
  draft,
  cut,
  children,
}: {
  draft: ImportDraft;
  /** What left it for size, in words, if anything did. */
  cut: string | null;
  /** What the owner can take out to make it again. */
  children?: ReactNode;
}) {
  const h = useT().maker.history;
  const swapped = KINDS.filter((kind) => (draft.redactions[kind] ?? 0) > 0).map((kind) =>
    h.kinds[kind](draft.redactions[kind] ?? 0),
  );
  return (
    <>
      <div className="flex items-start gap-3 rounded-xl border border-line bg-sunken px-4 py-3">
        <FileClock size={18} className="mt-0.5 shrink-0 text-muted" aria-hidden />
        <div className="flex flex-col gap-1">
          <p>{h.size(Math.max(1, Math.round(draft.tokensEstimate / 1000)))}</p>
          {swapped.length > 0 && (
            <p className="text-ink-soft">
              {h.swapped}: {swapped.join(", ")}
            </p>
          )}
          <p className="text-muted text-xs">{h.notPerfect}</p>
        </div>
      </div>
      {cut && <p className="text-ink-soft text-xs">{cut}</p>}
      {children}
      <pre
        className="max-h-[45vh] overflow-auto whitespace-pre-wrap rounded-xl border border-line bg-sunken p-4 font-mono text-xs leading-relaxed"
        data-selectable
      >
        {draft.text}
      </pre>
      <p className="text-muted text-xs leading-relaxed">{h.after}</p>
    </>
  );
}
