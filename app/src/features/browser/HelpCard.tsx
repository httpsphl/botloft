// A bot asking the owner for a hand in its browser (spec 21.10): what to
// do, on which site, and the way into the browser. Done and "I won't do
// it" answer the request; once answered it shrinks to one line.

import { Ban, Check, Hand, TimerOff } from "lucide-react";
import { useContext, useId } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { useAnswer } from "../chat/useAnswer";
import { ShowBrowser } from "./showBrowser";

/** The request the daemon opens from `browser_ask_owner`. */
export const HELP_TOOL = "mcp__botloft__browser_help";

/** The task in full, and the site, from the request's input. */
function read(approval: ApprovalItem): { task: string; site: string | null } {
  try {
    const parsed = JSON.parse(approval.input) as { task?: unknown; site?: unknown };
    return {
      task: typeof parsed.task === "string" && parsed.task ? parsed.task : approval.summary,
      site: typeof parsed.site === "string" ? parsed.site : null,
    };
  } catch {
    return { task: approval.summary, site: null };
  }
}

function Answered({ approval, task }: { approval: ApprovalItem; task: string }) {
  const words = useT().browser.help;
  const line = "flex min-w-0 items-center gap-2 rounded-lg px-1.5 py-1 text-sm";
  switch (approval.status) {
    case "allowed":
      return (
        <p className={line}>
          <Check aria-hidden size={14} className="shrink-0 text-ok" />
          <span className="truncate font-medium">{words.doneLine(task)}</span>
        </p>
      );
    case "denied":
      return (
        <p className={line}>
          <Ban aria-hidden size={14} className="shrink-0 text-danger" />
          <span className="truncate font-medium">{words.wontLine(task)}</span>
          {approval.note && <span className="truncate text-muted text-xs">“{approval.note}”</span>}
        </p>
      );
    default:
      return (
        <p className={line}>
          <TimerOff aria-hidden size={14} className="shrink-0 text-quiet" />
          <span className="truncate font-medium">{words.expired(task)}</span>
        </p>
      );
  }
}

export function HelpCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const words = t.browser.help;
  const showBrowser = useContext(ShowBrowser);
  const { note, setNote, busy, answer } = useAnswer(approval, {
    allow: t.chat.approval.allowFailed,
    deny: t.chat.approval.denyFailed,
  });
  const noteId = useId();
  const { task, site } = read(approval);
  if (approval.status !== "pending") {
    return <Answered approval={approval} task={task} />;
  }

  return (
    <section
      aria-label={words.asks(bot.name, task)}
      className="my-1 max-w-2xl rounded-2xl border border-accent/40 bg-panel px-4 py-3.5 shadow-sm animate-attention"
    >
      <p className="flex items-center gap-2.5 font-semibold text-sm">
        <span className="grid h-7 w-7 shrink-0 place-items-center rounded-full bg-accent/12 text-accent">
          <Hand aria-hidden size={15} />
        </span>
        {words.needs(bot.name)}
      </p>
      <p className="mt-2 break-words text-sm" data-selectable>
        {task}
      </p>
      {site && (
        <p className="mt-1 break-all font-mono text-ink-soft text-xs" data-selectable>
          {site}
        </p>
      )}
      <p className="mt-1.5 text-muted text-xs">{words.why}</p>
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
      <div className="mt-2.5 flex flex-wrap gap-2">
        {showBrowser && (
          <Button variant="primary" icon={Hand} onClick={() => showBrowser({ take: true })}>
            {words.take}
          </Button>
        )}
        <Button
          variant={showBrowser ? "secondary" : "primary"}
          icon={Check}
          disabled={busy}
          onClick={() => answer(true)}
        >
          {words.done}
        </Button>
        <Button variant="danger" icon={Ban} disabled={busy} onClick={() => answer(false)}>
          {words.wontDo}
        </Button>
      </div>
    </section>
  );
}
