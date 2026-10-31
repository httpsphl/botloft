// A request or a question in the island (spec 31.5): the agent's mascot,
// what it wants and the buttons. The phone's rules hold: a request cut to
// size or one that belongs to the computer is only denied here.

import { type CSSProperties, type FormEvent, type ReactNode, useRef, useState } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { toolTitle } from "../features/chat/toolNames";
import { useT } from "../i18n";
import type { BotloftApi } from "../lib/api";
import type { ApprovalCard, CardBot, QuestionCard } from "../lib/protocol.gen";

/** A card that just appeared ignores its buttons this long (spec 31.7). */
export const GUARD_MS = 600;

/** Runs `action` only once the card has been on screen for `GUARD_MS`; each
 *  card is its own component (keyed by its id), so the clock starts with it. */
function useGuard(): (action: () => void) => void {
  const shownAt = useRef(Date.now());
  return (action) => {
    if (Date.now() - shownAt.current >= GUARD_MS) {
      action();
    }
  };
}

function Who({ bot, crew }: { bot: CardBot; crew: string }) {
  return (
    <div className="flex w-32 shrink-0 flex-col items-center gap-2 pt-1 text-center">
      <span className="island-glow" style={{ "--glow": bot.color } as CSSProperties}>
        <BotAvatar color={bot.color} size={64} mood="waiting" still />
      </span>
      <span className="font-semibold text-[15px]">{bot.name}</span>
      <span className="island-muted text-xs">{crew}</span>
    </div>
  );
}

function Frame({
  bot,
  crew,
  title,
  pager,
  children,
}: {
  bot: CardBot;
  crew: string;
  title: string;
  pager: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className="island-card flex gap-4 p-4">
      <Who bot={bot} crew={crew} />
      <div className="flex min-w-0 flex-1 flex-col gap-2.5">
        <div className="flex items-center justify-between gap-3">
          <span className="font-semibold text-[15px]">{title}</span>
          {pager}
        </div>
        {children}
      </div>
    </div>
  );
}

export function RequestCard({
  api,
  card,
  pager,
  onOpen,
  onDone,
}: {
  api: BotloftApi;
  card: ApprovalCard;
  pager: ReactNode;
  onOpen: () => void;
  onDone: () => void;
}) {
  const t = useT();
  const guard = useGuard();
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);
  const answer = (allow: boolean) =>
    guard(() => {
      setBusy(true);
      setFailed(false);
      api
        .call("approvals.answer", { approvalId: card.approvalId, allow })
        .then(onDone, () => {
          setFailed(true);
          onDone();
        })
        .finally(() => setBusy(false));
    });
  const mayAllow = !card.cut && !card.atComputer;
  return (
    <Frame bot={card.bot} crew={card.crew} title={toolTitle(card.toolName, t.tools)} pager={pager}>
      {card.atComputer && (
        <p className="font-medium text-[13px] text-[var(--island-warn)]">
          {t.island.card.atComputer}
        </p>
      )}
      {card.explanation && <p className="island-soft text-[13px]">{card.explanation}</p>}
      <pre className="island-sunk max-h-[40vh] overflow-auto whitespace-pre-wrap break-all px-3 py-2.5 font-mono text-[12.5px] text-[var(--island-ink)]">
        {card.text}
      </pre>
      {card.cut && <p className="text-[12px] text-[var(--island-warn)]">{t.island.card.cut}</p>}
      {failed && <p className="text-[12px] text-[var(--island-danger)]">{t.island.card.failed}</p>}
      <div className="mt-1 flex gap-2">
        {mayAllow && (
          <button
            type="button"
            className="island-button"
            data-kind="allow"
            disabled={busy}
            onClick={() => answer(true)}
          >
            {t.island.card.allow}
          </button>
        )}
        <button
          type="button"
          className="island-button"
          data-kind="deny"
          disabled={busy}
          onClick={() => answer(false)}
        >
          {t.island.card.deny}
        </button>
        <button type="button" className="island-button ml-auto" data-kind="quiet" onClick={onOpen}>
          {t.island.card.openInApp}
        </button>
      </div>
    </Frame>
  );
}

export function AskCard({
  api,
  card,
  pager,
  onOpen,
  onDone,
}: {
  api: BotloftApi;
  card: QuestionCard;
  pager: ReactNode;
  onOpen: () => void;
  onDone: () => void;
}) {
  const t = useT();
  const guard = useGuard();
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);
  const send = (answer: string) =>
    guard(() => {
      if (answer.trim() === "") {
        return;
      }
      setBusy(true);
      setFailed(false);
      api
        .call("questions.answer", { questionId: card.questionId, answer: answer.trim() })
        .then(onDone, () => {
          setFailed(true);
          onDone();
        })
        .finally(() => setBusy(false));
    });
  const submit = (event: FormEvent) => {
    event.preventDefault();
    send(text);
  };
  return (
    <Frame bot={card.bot} crew={card.crew} title={t.island.card.asks(card.bot.name)} pager={pager}>
      <p className="whitespace-pre-wrap text-[13px] text-[var(--island-ink)]">{card.text}</p>
      {card.options.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {card.options.map((option) => (
            <button
              key={option}
              type="button"
              className="island-button"
              data-kind="option"
              disabled={busy}
              onClick={() => send(option)}
            >
              {option}
            </button>
          ))}
        </div>
      )}
      <form onSubmit={submit} className="island-field flex items-center gap-2 py-1.5 pr-1.5 pl-4">
        <input
          value={text}
          onChange={(event) => setText(event.target.value)}
          aria-label={t.island.card.answerLabel(card.bot.name)}
          placeholder={t.island.card.placeholder}
          className="min-w-0 flex-1 bg-transparent text-[13px] outline-none"
        />
        <button
          type="submit"
          className="island-button"
          data-kind="allow"
          disabled={busy || text.trim() === ""}
        >
          {t.island.card.send}
        </button>
      </form>
      {failed && <p className="text-[12px] text-[var(--island-danger)]">{t.island.card.failed}</p>}
      <div className="flex">
        <button type="button" className="island-button ml-auto" data-kind="quiet" onClick={onOpen}>
          {t.island.card.openInApp}
        </button>
      </div>
    </Frame>
  );
}
