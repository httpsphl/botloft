// An agent asking for one of the owner's skills (spec 38.5): the skill,
// why, and Give or Deny. Once answered it shrinks to one line.

import { Ban, Check, Sparkles, TimerOff } from "lucide-react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { RequestCard, SettledLine } from "../../ui/ChatCard";
import { RequestAnswers } from "../chat/RequestAnswers";
import { useAnswer } from "../chat/useAnswer";

/** The request `ask_skill` opens. */
export const SKILL_TOOL = "mcp__botloft__ask_skill";

function whyOf(input: string): string | null {
  try {
    const why = (JSON.parse(input) as { why?: unknown }).why;
    return typeof why === "string" && why.trim() !== "" ? why : null;
  } catch {
    return null;
  }
}

export function SkillRequestCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const words = t.skills.request;
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
  return (
    <RequestCard
      label={words.asks(bot.name, name)}
      icon={Sparkles}
      tone="accent"
      title={words.wants(bot.name, name)}
    >
      {why && (
        <p className="mt-2 break-words text-sm" data-selectable>
          {why}
        </p>
      )}
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
