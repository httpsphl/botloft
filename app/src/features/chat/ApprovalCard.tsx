// A permission request (spec 10.1): what the bot wants to do, with Allow
// and Deny. Once answered it shrinks to one line.

import { Ban, Check, Hand, TimerOff } from "lucide-react";
import { useId } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Details } from "../../ui/Details";
import { HELP_TOOL, HelpCard } from "../browser/HelpCard";
import { SITE_TOOL, SiteCard } from "../browser/SiteCard";
import { ROUTINE_TOOL, RoutineRequestCard } from "../routines/RoutineRequestCard";
import { commandOf, isCommand } from "./command";
import { PLAN_TOOL, PlanCard } from "./PlanCard";
import { SUGGEST_TOOL, SuggestionCard } from "./SuggestionCard";
import { pretty } from "./ToolLines";
import { toolAction } from "./toolNames";
import { useAnswer } from "./useAnswer";

/** What an answered request was about: the bot's words, or the command or file. */
function About({ approval }: { approval: ApprovalItem }) {
  return approval.explanation ? (
    <span className="truncate text-muted text-xs">{approval.explanation}</span>
  ) : (
    <span className="truncate font-mono text-muted text-xs">{approval.summary}</span>
  );
}

function Answered({ approval }: { approval: ApprovalItem }) {
  const t = useT();
  const label = toolAction(approval.toolName, t.tools);
  const line = "flex min-w-0 items-center gap-2 rounded-lg px-1.5 py-1 text-sm";
  switch (approval.status) {
    case "allowed":
      return (
        <p className={line}>
          <Check aria-hidden size={14} className="shrink-0 text-ok" />
          <span className="shrink-0 font-medium">{t.chat.approval.allowed(label)}</span>
          <About approval={approval} />
        </p>
      );
    case "denied":
      return (
        <p className={line}>
          <Ban aria-hidden size={14} className="shrink-0 text-danger" />
          <span className="shrink-0 font-medium">{t.chat.approval.denied(label)}</span>
          {approval.note ? (
            <span className="truncate text-muted text-xs">{`“${approval.note}”`}</span>
          ) : (
            <About approval={approval} />
          )}
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
    case ROUTINE_TOOL:
      return <RoutineRequestCard approval={approval} bot={bot} />;
    case SITE_TOOL:
      return <SiteCard approval={approval} bot={bot} />;
    case HELP_TOOL:
      return <HelpCard approval={approval} bot={bot} />;
    default:
      return <ToolApproval approval={approval} bot={bot} />;
  }
}

const BOX =
  "max-h-56 overflow-auto break-all rounded-lg border border-line bg-sunken px-2.5 py-1.5";

/**
 * A command the bot wants to run: first what the bot says it is for, in
 * plain text, then the command itself, one click away. The bot wrote the
 * explanation, so the card says so and never shows it in the command's
 * place; with none, the command is open from the start (spec 10.1).
 */
function CommandAsked({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const said = approval.explanation;
  const command = commandOf(approval.input);
  return (
    <>
      {said ? (
        <>
          <p className="mt-2 break-words text-sm" data-selectable>
            {said}
          </p>
          <p className="mt-1 text-muted text-xs">{t.chat.approval.explainedBy(bot.name)}</p>
        </>
      ) : (
        <p className="mt-2 text-ink-soft text-sm">{t.chat.approval.unexplained(bot.name)}</p>
      )}
      <Details label={t.chat.approval.command} open={!said}>
        <div className={BOX}>{command.text}</div>
        {!command.whole && <p className="mt-1 font-sans text-warn">{t.chat.tools.commandCut}</p>}
      </Details>
    </>
  );
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
  const label = toolAction(approval.toolName, t.tools);

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
      {isCommand(approval.toolName) ? (
        <CommandAsked approval={approval} bot={bot} />
      ) : (
        <>
          {approval.summary && (
            <p className="mt-2 break-words font-mono text-ink-soft text-xs" data-selectable>
              {approval.summary}
            </p>
          )}
          <Details>
            <div className={BOX}>{pretty(approval.input)}</div>
          </Details>
        </>
      )}
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
