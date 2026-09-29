// A permission request (spec 10.1): what the bot wants to do, with Allow
// and Deny. Once answered it shrinks to one line.

import { Ban, Check, Hand, TimerOff } from "lucide-react";
import { useId, useState } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { pretty, toolLabel } from "./ToolLines";

function Answered({ approval }: { approval: ApprovalItem }) {
  const t = useT();
  const label = toolLabel(approval.toolName);
  const line = "flex min-w-0 items-center gap-2 px-1.5 py-1 text-sm";
  switch (approval.status) {
    case "allowed":
      return (
        <p className={line}>
          <Check aria-hidden size={14} className="shrink-0 text-ok" />
          <span className="shrink-0 font-medium">{t.chat.approval.allowed(label)}</span>
          <span className="truncate font-mono text-muted text-xs">{approval.summary}</span>
        </p>
      );
    case "denied":
      return (
        <p className={line}>
          <Ban aria-hidden size={14} className="shrink-0 text-danger" />
          <span className="shrink-0 font-medium">{t.chat.approval.denied(label)}</span>
          <span className="truncate text-muted text-xs">
            {approval.note ? `“${approval.note}”` : approval.summary}
          </span>
        </p>
      );
    default:
      return (
        <p className={line}>
          <TimerOff aria-hidden size={14} className="shrink-0 text-quiet" />
          <span className="font-medium">{t.chat.approval.expired(label)}</span>
        </p>
      );
  }
}

export function ApprovalCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const api = useApi();
  const t = useT();
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  const noteId = useId();
  if (approval.status !== "pending") {
    return <Answered approval={approval} />;
  }
  const label = toolLabel(approval.toolName);
  const answer = async (allow: boolean) => {
    setBusy(true);
    const trimmed = note.trim();
    await attempt(allow ? t.chat.approval.allowFailed : t.chat.approval.denyFailed, () =>
      api.call("approvals.answer", {
        approvalId: approval.approvalId,
        allow,
        ...(allow || !trimmed ? {} : { note: trimmed }),
      }),
    );
    setBusy(false);
  };

  return (
    <section
      aria-label={t.chat.approval.asks(bot.name, label)}
      className="my-1 max-w-2xl rounded-md border border-warn/60 border-l-4 bg-panel px-3.5 py-3"
    >
      <p className="flex items-center gap-2 font-semibold text-sm">
        <Hand aria-hidden size={15} className="text-warn" />
        {t.chat.approval.wants(bot.name, label)}
      </p>
      {approval.summary && (
        <p className="mt-1.5 break-words font-mono text-ink-soft text-xs" data-selectable>
          {approval.summary}
        </p>
      )}
      <details className="mt-1.5 text-xs">
        <summary className="cursor-default text-muted hover:text-ink">
          {t.chat.approval.fullInput}
        </summary>
        <pre
          className="mt-1 max-h-56 overflow-auto whitespace-pre-wrap break-all rounded-[3px] border border-line bg-sunken px-2.5 py-1.5 font-mono"
          data-selectable
        >
          {pretty(approval.input)}
        </pre>
      </details>
      <label htmlFor={noteId} className="sr-only">
        {t.chat.approval.noteLabel(bot.name)}
      </label>
      <input
        id={noteId}
        value={note}
        onChange={(event) => setNote(event.target.value)}
        placeholder={t.chat.approval.notePlaceholder}
        className="mt-3 h-8 w-full rounded-[3px] border border-line-strong bg-canvas px-2.5 text-sm outline-none placeholder:text-muted focus:border-accent"
      />
      <div className="mt-2.5 flex gap-2">
        <Button variant="primary" icon={Check} disabled={busy} onClick={() => answer(true)}>
          {t.chat.approval.allow}
        </Button>
        <Button variant="danger" icon={Ban} disabled={busy} onClick={() => answer(false)}>
          {t.chat.approval.deny}
        </Button>
      </div>
    </section>
  );
}
