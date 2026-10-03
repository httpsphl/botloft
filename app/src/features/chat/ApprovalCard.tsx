// A permission request (spec 10.1): what the bot wants to do, with Allow
// and Deny. Once answered it shrinks to one line.

import { Ban, Check, CheckCheck, Hand, TimerOff } from "lucide-react";
import { useT } from "../../i18n";
import type { AllowScope, ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { NoteInput, RequestCard, SettledLine } from "../../ui/ChatCard";
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
  switch (approval.status) {
    case "allowed":
      return (
        <SettledLine
          icon={Check}
          tone="ok"
          text={t.chat.approval.allowed(label)}
          aside={<About approval={approval} />}
        />
      );
    case "denied":
      return (
        <SettledLine
          icon={Ban}
          tone="danger"
          text={t.chat.approval.denied(label)}
          note={approval.note}
          aside={approval.note ? undefined : <About approval={approval} />}
        />
      );
    default:
      return <SettledLine icon={TimerOff} tone="quiet" text={t.chat.approval.expired(label)} />;
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

/** "Allow always", by what it would cover. */
function alwaysLabel(
  scope: AllowScope,
  words: ReturnType<typeof useT>["chat"]["approval"]["always"],
): string {
  switch (scope.kind) {
    case "command":
      return words.command;
    case "site":
      return words.site(scope.value);
    case "file":
      return words.file;
    default:
      return words.tool;
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
  if (approval.status !== "pending") {
    return <Answered approval={approval} />;
  }
  const label = toolAction(approval.toolName, t.tools);

  return (
    <RequestCard
      label={t.chat.approval.asks(bot.name, label)}
      icon={Hand}
      tone="warn"
      title={t.chat.approval.wants(bot.name, label)}
    >
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
      <NoteInput
        label={t.chat.approval.noteLabel(bot.name)}
        value={note}
        onChange={setNote}
        placeholder={t.chat.approval.notePlaceholder}
      />
      <div className="mt-2.5 flex flex-wrap gap-2">
        <Button variant="primary" icon={Check} disabled={busy} onClick={() => answer(true)}>
          {t.chat.approval.allow}
        </Button>
        {approval.always && (
          <Button
            icon={CheckCheck}
            disabled={busy}
            title={t.chat.approval.always.hint(bot.name)}
            onClick={() => answer(true, undefined, true)}
          >
            {alwaysLabel(approval.always, t.chat.approval.always)}
          </Button>
        )}
        <Button variant="danger" icon={Ban} disabled={busy} onClick={() => answer(false)}>
          {t.chat.approval.deny}
        </Button>
      </div>
    </RequestCard>
  );
}
