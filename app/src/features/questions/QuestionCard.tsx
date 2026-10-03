// A question a bot asked the owner (spec 23.6): the question, ready answers
// as buttons and a field for any other answer. The same card is in the
// bot's chat and in the question box. Once closed it shrinks to one line.

import { Check, MessageCircleQuestion, Reply, X } from "lucide-react";
import { useId, useState } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { Question } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { Markdown } from "../chat/Markdown";

export function QuestionCard({ question, bot }: { question: Question; bot: string }) {
  if (question.status !== "open") {
    return <Closed question={question} />;
  }
  return <OpenQuestion question={question} bot={bot} />;
}

function OpenQuestion({ question, bot }: { question: Question; bot: string }) {
  const t = useT();
  const q = t.questions.card;
  const api = useApi();
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const fieldId = useId();

  const answer = async (value: string) => {
    setBusy(true);
    const sent = await attempt(q.answerFailed, () =>
      api.call("questions.answer", { questionId: question.id, answer: value }),
    );
    setBusy(false);
    if (sent) {
      setText("");
    }
  };
  const dismiss = async () => {
    setBusy(true);
    await attempt(q.dismissFailed, () =>
      api.call("questions.dismiss", { questionId: question.id }),
    );
    setBusy(false);
  };

  return (
    <section
      aria-label={q.asks(bot)}
      className="my-1 overflow-hidden rounded-2xl border border-line bg-panel shadow-sm animate-attention"
    >
      <p className="flex items-center gap-2.5 border-line border-b px-4 py-3 font-semibold text-sm">
        <span className="grid h-7 w-7 place-items-center rounded-full bg-warn/12 text-warn">
          <MessageCircleQuestion aria-hidden size={15} />
        </span>
        {q.asks(bot)}
      </p>
      <div className="flex flex-col gap-3 px-4 py-3" data-selectable>
        <Markdown text={question.text} />
        {question.options.length > 0 && (
          <fieldset aria-label={q.pick} className="flex flex-wrap gap-2">
            {question.options.map((option) => (
              <Button key={option} disabled={busy} onClick={() => answer(option)}>
                {option}
              </Button>
            ))}
          </fieldset>
        )}
      </div>
      <form
        className="border-line border-t bg-canvas/40 px-4 py-3"
        onSubmit={(event) => {
          event.preventDefault();
          if (text.trim()) {
            void answer(text);
          }
        }}
      >
        <label htmlFor={fieldId} className="sr-only">
          {q.answerLabel(bot)}
        </label>
        <textarea
          id={fieldId}
          rows={2}
          value={text}
          onChange={(event) => setText(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && !event.shiftKey && text.trim()) {
              event.preventDefault();
              void answer(text);
            }
          }}
          placeholder={q.placeholder}
          className="block w-full resize-none rounded-xl border border-line-strong bg-panel px-3 py-2 text-sm outline-none placeholder:text-muted focus:border-muted"
        />
        <div className="mt-2.5 flex flex-wrap gap-2">
          <Button type="submit" variant="primary" icon={Reply} disabled={busy || !text.trim()}>
            {q.send}
          </Button>
          <Button
            type="button"
            variant="ghost"
            icon={X}
            disabled={busy}
            title={q.dismissHint(bot)}
            onClick={dismiss}
          >
            {q.dismiss}
          </Button>
        </div>
      </form>
    </section>
  );
}

/** One line: what the owner answered, or that they dismissed it. */
function Closed({ question }: { question: Question }) {
  const t = useT();
  const q = t.questions.card;
  const answered = question.status === "answered";
  return (
    <details className="group">
      <summary className="flex min-w-0 cursor-default items-center gap-2 rounded-lg px-1.5 py-1 text-sm hover:bg-sunken">
        {answered ? (
          <Check aria-hidden size={14} className="shrink-0 text-ok" />
        ) : (
          <X aria-hidden size={14} className="shrink-0 text-quiet" />
        )}
        <span className="shrink-0 font-medium">{answered ? q.answered : q.dismissed}</span>
        {answered && question.answer && (
          <span className="truncate text-muted text-xs">{`“${question.answer}”`}</span>
        )}
        {question.answeredAt !== null && (
          <time
            className="ml-auto shrink-0 text-muted text-xs"
            dateTime={new Date(question.answeredAt).toISOString()}
          >
            {when(question.answeredAt)}
          </time>
        )}
      </summary>
      <div
        className="mt-1.5 ml-6 flex flex-col gap-2 rounded-xl border border-line bg-panel px-4 py-3 text-sm"
        data-selectable
      >
        <Markdown text={question.text} />
        {answered && question.answer && (
          <p className="whitespace-pre-wrap border-line border-t pt-2 text-ink-soft">
            {question.answer}
          </p>
        )}
      </div>
    </details>
  );
}
