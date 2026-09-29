// A permission request (spec 10.1): what the bot wants to do, with Allow
// and Deny. Once answered it shrinks to one line.

import { Ban, Check, Hand, TimerOff } from "lucide-react";
import { useId } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { HELP_TOOL, HelpCard } from "../browser/HelpCard";
import { SITE_TOOL, SiteCard } from "../browser/SiteCard";
import { PLAN_TOOL, PlanCard } from "./PlanCard";
import { SUGGEST_TOOL, SuggestionCard } from "./SuggestionCard";
import { pretty, toolLabel } from "./ToolLines";
import { useAnswer } from "./useAnswer";

function Answered({ approval }: { approval: ApprovalItem }) {
  const t = useT();
  const label = toolLabel(approval.toolName);
  const line = "flex min-w-0 items-center gap-2 rounded-lg px-1.5 py-1 text-sm";
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
  switch (approval.toolName) {
    case PLAN_TOOL:
      return <PlanCard approval={approval} bot={bot} />;
    case SUGGEST_TOOL:
      return <SuggestionCard approval={approval} bot={bot} />;
    case SITE_TOOL:
      return <SiteCard approval={approval} bot={bot} />;
    case HELP_TOOL:
      return <HelpCard approval={approval} bot={bot} />;
    default:
      return <ToolApproval approval={approval} bot={bot} />;
  }
}

function ToolApproval({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const { note, setNote, busy, answer } = useAnswer(approval, {
    allow: t.chat.approval.allowFailed,
    deny: t.chat.approval.denyFailed,
  });
  const noteId = useId();
  if (approval.status !== "pending") {
    return <Answered approval={approval} />;
  }
  const label = toolLabel(approval.toolName);

  return (
    <section
      aria-label={t.chat.approval.asks(bot.name, label)}
      className="my-1 max-w-2xl rounded-2xl border border-warn/40 bg-panel px-4 py-3.5 shadow-sm animate-attention"
    >
      <p className="flex items-center gap-2.5 font-semibold text-sm">
        <span className="grid h-7 w-7 shrink-0 place-items-center rounded-full bg-warn/12 text-warn">
          <Hand aria-hidden size={15} />
        </span>
        {t.chat.approval.wants(bot.name, label)}
      </p>
      {approval.summary && (
        <p className="mt-2 break-words font-mono text-ink-soft text-xs" data-selectable>
          {approval.summary}
        </p>
      )}
      <details className="mt-2 text-xs">
        <summary className="cursor-default text-muted hover:text-ink">
          {t.chat.approval.fullInput}
        </summary>
        <pre
          className="mt-1.5 max-h-56 overflow-auto whitespace-pre-wrap break-all rounded-lg border border-line bg-sunken px-2.5 py-1.5 font-mono"
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
        className="mt-3 h-9 w-full rounded-xl border border-line-strong bg-canvas px-3 text-sm outline-none placeholder:text-muted focus:border-muted"
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
