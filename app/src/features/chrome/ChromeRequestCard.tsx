// An agent asking for the owner's own Chrome (spec 37.5): why, the risks,
// and Give only once the owner checked that they understand them. Once
// answered it shrinks to one line.

import { AppWindow, Ban, Check, TimerOff } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { RequestCard, SettledLine } from "../../ui/ChatCard";
import { RequestAnswers } from "../chat/RequestAnswers";
import { useAnswer } from "../chat/useAnswer";
import { ChromeRisks } from "./ChromeRisks";

/** The request `ask_chrome` opens. */
export const CHROME_TOOL = "mcp__botloft__ask_chrome";

export function ChromeRequestCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const words = t.chrome.request;
  const [understood, setUnderstood] = useState(false);
  const { note, setNote, busy, answer } = useAnswer(approval, {
    allow: t.chat.approval.allowFailed,
    deny: t.chat.approval.denyFailed,
  });
  if (approval.status === "allowed") {
    return <SettledLine icon={Check} tone="ok" text={words.given(bot.name)} />;
  }
  if (approval.status === "denied") {
    return <SettledLine icon={Ban} tone="danger" text={words.declined} note={approval.note} />;
  }
  if (approval.status !== "pending") {
    return <SettledLine icon={TimerOff} tone="quiet" text={words.expired} />;
  }
  return (
    <RequestCard
      label={words.asks(bot.name)}
      icon={AppWindow}
      tone="accent"
      title={words.wants(bot.name)}
    >
      {approval.summary && (
        <p className="mt-2 break-words text-sm" data-selectable>
          {approval.summary}
        </p>
      )}
      <ChromeRisks bot={bot.name} understood={understood} onUnderstood={setUnderstood} />
      <RequestAnswers
        bot={bot.name}
        deny={t.chat.approval.deny}
        busy={busy}
        note={note}
        onNote={setNote}
        onDeny={() => answer(false)}
      >
        <Button
          variant="primary"
          icon={Check}
          disabled={busy || !understood}
          onClick={() => answer(true)}
        >
          {words.give}
        </Button>
      </RequestAnswers>
    </RequestCard>
  );
}
