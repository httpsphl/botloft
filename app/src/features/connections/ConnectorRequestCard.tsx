// An agent asking for a tool of the owner's claude.ai account (spec 34.6),
// instead of signing in to the service in its browser: the tool, why, and
// Allow or Deny. Once answered it shrinks to one line.

import { Ban, Check, Plug, TimerOff } from "lucide-react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { RequestCard, SettledLine } from "../../ui/ChatCard";
import { RequestAnswers } from "../chat/RequestAnswers";
import { useAnswer } from "../chat/useAnswer";
import { ToolTile } from "./ToolTile";
import { knownTool } from "./toolCatalog";

/** The request `ask_connector` opens. */
export const CONNECTOR_TOOL = "mcp__botloft__ask_connector";

function whyOf(input: string): string | null {
  try {
    const parsed: unknown = JSON.parse(input);
    const why = (parsed as { why?: unknown }).why;
    return typeof why === "string" && why.trim() !== "" ? why : null;
  } catch {
    return null;
  }
}

export function ConnectorRequestCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const words = t.connections.request;
  const { note, setNote, busy, answer } = useAnswer(approval, {
    allow: t.chat.approval.allowFailed,
    deny: t.chat.approval.denyFailed,
  });
  const name = approval.summary;
  if (approval.status === "allowed") {
    return <SettledLine icon={Check} tone="ok" text={words.given(name, bot.name)} />;
  }
  if (approval.status === "denied") {
    return (
      <SettledLine icon={Ban} tone="danger" text={words.declined(name)} note={approval.note} />
    );
  }
  if (approval.status !== "pending") {
    return <SettledLine icon={TimerOff} tone="quiet" text={words.expired(name)} />;
  }
  const why = whyOf(approval.input);
  const known = knownTool(name);
  return (
    <RequestCard
      label={words.asks(bot.name, name)}
      icon={Plug}
      tone="accent"
      title={words.wants(bot.name, name)}
    >
      <div className="mt-2.5 flex items-start gap-3">
        <ToolTile name={name} size={36} />
        <div className="flex min-w-0 flex-col gap-1 text-sm">
          {known && <p className="text-ink-soft">{t.connections.about[known.key]}</p>}
          {why && (
            <p className="break-words" data-selectable>
              {why}
            </p>
          )}
          <p className="text-muted text-xs">{words.how(bot.name)}</p>
        </div>
      </div>
      <RequestAnswers
        bot={bot.name}
        deny={t.chat.approval.deny}
        busy={busy}
        note={note}
        onNote={setNote}
        onDeny={() => answer(false)}
      >
        <Button variant="primary" icon={Check} disabled={busy} onClick={() => answer(true)}>
          {words.give}
        </Button>
      </RequestAnswers>
    </RequestCard>
  );
}
