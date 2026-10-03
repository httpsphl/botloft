// The question box (spec 23.6): every question the bots asked and the owner
// has not answered, newest first, each with its card to answer right here.

import { MessageCircleQuestion, MessageSquare } from "lucide-react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import { AVATAR_PALETTE, type Question } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { openQuestions } from "../../store/questions";
import { Button } from "../../ui/Button";
import { EmptyState } from "../../ui/EmptyState";
import { BotAvatar } from "../bots/BotAvatar";
import { QuestionCard } from "./QuestionCard";

export function QuestionBox() {
  const t = useT();
  const words = t.questions.box;
  const questions = useApp(openQuestions);
  return (
    <section aria-label={words.label} className="flex min-h-0 flex-1 flex-col">
      <header className="border-line border-b px-5 py-3.5">
        <h1 className="truncate font-semibold text-xl tracking-tight">{words.title}</h1>
        <p className="mt-0.5 text-muted text-sm">{words.intro}</p>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto p-5">
        {questions.length === 0 ? (
          <Empty />
        ) : (
          <ul className="mx-auto flex max-w-3xl flex-col gap-6">
            {questions.map((question) => (
              <Entry key={question.id} question={question} />
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}

function Entry({ question }: { question: Question }) {
  const t = useT();
  const words = t.questions.box;
  const bot = useApp((state) => state.bots[question.botId]);
  const crew = useApp((state) => state.crews[question.crewId]);
  const selectBot = useApp((state) => state.selectBot);
  if (!bot) {
    return null;
  }
  return (
    <li className="flex gap-3">
      <BotAvatar color={bot.color} size={28} still />
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <div className="flex items-center gap-2 text-sm">
          <span className="font-semibold">{bot.name}</span>
          {crew && <span className="truncate text-muted text-xs">{words.inCrew(crew.name)}</span>}
          <time
            className="text-muted text-xs"
            dateTime={new Date(question.createdAt).toISOString()}
          >
            {when(question.createdAt)}
          </time>
          <Button
            variant="ghost"
            size="sm"
            icon={MessageSquare}
            label={words.openChat(bot.name)}
            className="ml-auto"
            onClick={() => selectBot(bot.id)}
          />
        </div>
        <QuestionCard question={question} bot={bot.name} />
      </div>
    </li>
  );
}

function Empty() {
  const words = useT().questions.box;
  return (
    <div className="mt-10">
      <EmptyState
        framed
        color={AVATAR_PALETTE[0]}
        icon={MessageCircleQuestion}
        title={words.emptyTitle}
        body={words.emptyBody}
      />
    </div>
  );
}
