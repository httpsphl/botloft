// A bot asking the owner for a hand in its browser (spec 21.10): what to
// do, on which site, and the way into the browser. Done and "I won't do
// it" answer the request; once answered it shrinks to one line.

import { Ban, Check, Hand, TimerOff } from "lucide-react";
import { useContext } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { NoteInput, RequestCard, SettledLine } from "../../ui/ChatCard";
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
  switch (approval.status) {
    case "allowed":
      return <SettledLine icon={Check} tone="ok" text={words.doneLine(task)} />;
    case "denied":
      return (
        <SettledLine icon={Ban} tone="danger" text={words.wontLine(task)} note={approval.note} />
      );
    default:
      return <SettledLine icon={TimerOff} tone="quiet" text={words.expired(task)} />;
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
  const { task, site } = read(approval);
  if (approval.status !== "pending") {
    return <Answered approval={approval} task={task} />;
  }

  return (
    <RequestCard
      label={words.asks(bot.name, task)}
      icon={Hand}
      tone="accent"
      title={words.needs(bot.name)}
    >
      <p className="mt-2 break-words text-sm" data-selectable>
        {task}
      </p>
      {site && (
        <p className="mt-1 break-all font-mono text-ink-soft text-xs" data-selectable>
          {site}
        </p>
      )}
      <p className="mt-1.5 text-muted text-xs">{words.why}</p>
      <NoteInput
        label={t.chat.approval.noteLabel(bot.name)}
        value={note}
        onChange={setNote}
        placeholder={t.chat.approval.notePlaceholder}
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
    </RequestCard>
  );
}
