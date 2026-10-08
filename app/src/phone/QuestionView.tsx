// A question waiting for the owner (spec 23, 28.7): the text, the ready
// answers as buttons, a field for another one, and Dismiss.

import { useState } from "react";
import { useT } from "../i18n";
import { when } from "../lib/format";
import type { QuestionCard } from "../lib/protocol.gen";
import { BotDot, PhoneButton, PhoneMarkdown } from "./parts";

export function QuestionView({
  card,
  sending,
  answer,
  dismiss,
}: {
  card: QuestionCard;
  sending: boolean;
  /** False if the answer could not be sent. */
  answer(text: string): Promise<boolean>;
  dismiss(): Promise<boolean>;
}) {
  const t = useT().phone;
  const q = t.question;
  const [text, setText] = useState("");
  const [failed, setFailed] = useState(false);

  const run = async (work: () => Promise<boolean>) => {
    setFailed(false);
    setFailed(!(await work()));
  };

  return (
    <article className="flex flex-col gap-3 rounded-2xl border border-line bg-panel p-4">
      <header className="flex items-center gap-3">
        <BotDot name={card.bot.name} color={card.bot.color} />
        <div className="min-w-0">
          <p className="truncate font-semibold">{q.asks(card.bot.name)}</p>
          <p className="truncate text-muted text-xs">
            {t.inbox.inCrew(card.crew)} · {when(card.createdAt)}
          </p>
        </div>
      </header>

      <PhoneMarkdown>{card.text}</PhoneMarkdown>

      {card.options.length > 0 && (
        <fieldset className="m-0 flex min-w-0 flex-col gap-2 border-0 p-0">
          <legend className="sr-only">{q.pick}</legend>
          {card.options.map((option) => (
            <PhoneButton
              key={option}
              disabled={sending}
              onClick={() => run(() => answer(option))}
              className="flex-none"
            >
              {option}
            </PhoneButton>
          ))}
        </fieldset>
      )}

      <label className="flex flex-col gap-1 text-muted text-sm">
        {q.answerLabel}
        <textarea
          value={text}
          rows={2}
          placeholder={q.placeholder}
          onChange={(event) => setText(event.target.value)}
          className="rounded-xl border border-line-strong bg-canvas px-3 py-2 text-base text-ink"
        />
      </label>
      <div className="flex gap-3">
        <PhoneButton
          look="primary"
          disabled={sending || text.trim() === ""}
          onClick={() => run(() => answer(text.trim()))}
        >
          {sending ? q.sending : q.send}
        </PhoneButton>
        <PhoneButton disabled={sending} onClick={() => run(dismiss)}>
          {q.dismiss}
        </PhoneButton>
      </div>
      <p className="text-muted text-xs">{q.dismissHint(card.bot.name)}</p>
      {failed && (
        <p role="alert" className="text-danger text-sm">
          {q.failed}
        </p>
      )}
    </article>
  );
}
