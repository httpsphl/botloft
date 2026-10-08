// A request waiting for the owner, as the phone shows it (spec 28.5, 28.7):
// what the bot wants, what it says it is for, the whole request a tap away,
// and Allow or Deny. Allow is off for what is cut or belongs to the computer.

import { useState } from "react";
import { useT } from "../i18n";
import { when } from "../lib/format";
import type { ApprovalCard } from "../lib/protocol.gen";
import { Callout } from "../ui/Callout";
import { BotDot, PhoneButton } from "./parts";

export function ApprovalView({
  card,
  sending,
  answer,
}: {
  card: ApprovalCard;
  sending: boolean;
  /** False if the answer could not be sent. */
  answer(allow: boolean, note?: string): Promise<boolean>;
}) {
  const t = useT().phone;
  const a = t.approval;
  const [all, setAll] = useState(false);
  const [note, setNote] = useState("");
  const [failed, setFailed] = useState(false);
  const [pressed, setPressed] = useState<boolean | null>(null);

  const send = async (allow: boolean) => {
    setFailed(false);
    setPressed(allow);
    const sent = await answer(allow, note);
    setFailed(!sent);
  };
  const canAllow = !card.cut && !card.atComputer;

  return (
    <article className="flex flex-col gap-3 rounded-2xl border border-line bg-panel p-4">
      <header className="flex items-center gap-3">
        <BotDot name={card.bot.name} color={card.bot.color} />
        <div className="min-w-0">
          <p className="truncate font-semibold">{card.bot.name}</p>
          <p className="truncate text-muted text-xs">
            {t.inbox.inCrew(card.crew)} · {when(card.createdAt)}
          </p>
        </div>
      </header>

      <div>
        <p className="text-muted text-sm">{a.asks(card.bot.name)}</p>
        <p className="break-words font-medium text-base">{card.summary || card.toolName}</p>
      </div>

      {card.explanation ? (
        <div className="rounded-xl bg-sunken px-3 py-2.5">
          <p className="whitespace-pre-wrap break-words text-base">{card.explanation}</p>
          <p className="mt-1 text-muted text-xs">{a.explanationBy(card.bot.name)}</p>
        </div>
      ) : (
        canAllow && <Callout tone="warn" title={a.noExplanation(card.bot.name)} />
      )}

      <div>
        <button
          type="button"
          aria-expanded={all}
          onClick={() => setAll(!all)}
          className="min-h-11 font-medium text-work text-sm underline underline-offset-2"
        >
          {all ? a.hideAll : a.showAll}
        </button>
        {all && (
          <pre
            data-selectable
            className="mt-1 max-h-72 overflow-auto whitespace-pre-wrap break-words rounded-xl bg-sunken p-3 font-mono text-sm"
          >
            {card.text}
          </pre>
        )}
      </div>

      {card.cut && <Callout tone="warn" title={a.cut} />}
      {!card.cut && card.atComputer && <Callout tone="warn" title={a.atComputer} />}

      <label className="flex flex-col gap-1 text-muted text-sm">
        {a.noteLabel}
        <input
          value={note}
          maxLength={500}
          onChange={(event) => setNote(event.target.value)}
          className="h-12 rounded-xl border border-line-strong bg-canvas px-3 text-base text-ink"
        />
      </label>

      <div className="flex gap-3">
        <PhoneButton look="primary" disabled={!canAllow || sending} onClick={() => send(true)}>
          {sending && pressed === true ? a.sending : a.allow}
        </PhoneButton>
        <PhoneButton look="danger" disabled={sending} onClick={() => send(false)}>
          {sending && pressed === false ? a.sending : a.deny}
        </PhoneButton>
      </div>
      {failed && (
        <p role="alert" className="text-danger text-sm">
          {a.failed}
        </p>
      )}
    </article>
  );
}
