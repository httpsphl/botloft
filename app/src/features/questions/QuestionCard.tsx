// A question a bot asked the owner (spec 23.6): the question, ready answers
// as buttons and a field for any other answer. A note it left them (23.10)
// asks nothing: a field to reply, if they want, and Seen. The same card is
// in the bot's chat, the inbox and the home. Once closed it shrinks to one
// line.

import { Check, Eye, MessageCircleQuestion, Reply, StickyNote, X } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { Question } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { NoteArea, SectionCard, SettledLine } from "../../ui/ChatCard";
import { attempt } from "../../ui/toast";
import { Markdown } from "../chat/Markdown";

/** `title` replaces "<bot> asks" at the card's head where the bot's name is already above it. */
export function QuestionCard({
  question,
  bot,
  title,
}: {
  question: Question;
  bot: string;
  title?: string;
}) {
  if (question.status !== "open") {
    return <Closed question={question} />;
  }
  return <OpenQuestion question={question} bot={bot} title={title} />;
}

function OpenQuestion({
  question,
  bot,
  title,
}: {
  question: Question;
  bot: string;
  title: string | undefined;
}) {
  const t = useT();
  const q = t.questions.card;
  const api = useApi();
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const note = question.kind === "note";

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
  const answerText = () => {
    if (text.trim()) {
      void answer(text);
    }
  };
  const dismiss = async () => {
    setBusy(true);
    await attempt(note ? q.seenFailed : q.dismissFailed, () =>
      api.call("questions.dismiss", { questionId: question.id }),
    );
    setBusy(false);
  };

  return (
    <SectionCard
      label={note ? q.tells(bot) : q.asks(bot)}
      title={title}
      icon={note ? StickyNote : MessageCircleQuestion}
      tone={note ? "accent" : "warn"}
      footer={
        <form
          onSubmit={(event) => {
            event.preventDefault();
            answerText();
          }}
        >
          <NoteArea
            label={note ? q.replyLabel(bot) : q.answerLabel(bot)}
            value={text}
            onChange={setText}
            onEnter={answerText}
            placeholder={note ? q.replyPlaceholder : q.placeholder}
          />
          <div className="mt-2.5 flex flex-wrap gap-2">
            <Button type="submit" variant="primary" icon={Reply} disabled={busy || !text.trim()}>
              {note ? q.reply : q.send}
            </Button>
            <Button
              type="button"
              variant="ghost"
              icon={note ? Check : X}
              disabled={busy}
              title={note ? q.seenHint(bot) : q.dismissHint(bot)}
              onClick={dismiss}
            >
              {note ? q.seen : q.dismiss}
            </Button>
          </div>
        </form>
      }
    >
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
    </SectionCard>
  );
}

/** One line: what the owner answered, or that they dismissed it (a note: saw it). */
function Closed({ question }: { question: Question }) {
  const q = useT().questions.card;
  const answered = question.status === "answered";
  const note = question.kind === "note";
  const said = note ? (answered ? q.replied : q.wasSeen) : answered ? q.answered : q.dismissed;
  return (
    <SettledLine
      icon={answered ? Check : note ? Eye : X}
      tone={answered ? "ok" : "quiet"}
      text={said}
      note={answered ? question.answer : null}
      aside={
        question.answeredAt !== null && (
          <time
            className="ml-auto shrink-0 text-muted text-xs"
            dateTime={new Date(question.answeredAt).toISOString()}
          >
            {when(question.answeredAt)}
          </time>
        )
      }
      details={
        <>
          <Markdown text={question.text} />
          {answered && question.answer && (
            <p className="whitespace-pre-wrap border-line border-t pt-2 text-ink-soft">
              {question.answer}
            </p>
          )}
        </>
      }
    />
  );
}
