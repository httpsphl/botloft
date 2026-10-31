// "Needs you" on the home (spec 15.1): the requests and questions of every
// crew, oldest first, each answered right here; then messages that were not
// delivered and agents whose engine is signed out.

import { CircleX, KeyRound } from "lucide-react";
import { type ReactNode, useState } from "react";
import { useT } from "../../i18n";
import {
  type ApprovalCard,
  AVATAR_PALETTE,
  type Bot,
  type Delivery,
  type Question,
} from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { EmptyState } from "../../ui/EmptyState";
import { FailedDialog } from "../messages/FailedDeliveries";
import { QuestionEntry } from "../questions/QuestionBox";
import { RequestEntry } from "./RequestEntry";

type Entry =
  | { kind: "request"; at: number; card: ApprovalCard }
  | { kind: "question"; at: number; question: Question };

export function NeedsYou({
  requests,
  questions,
  dead,
  signedOut,
  onAnswered,
}: {
  requests: ApprovalCard[];
  questions: Question[];
  dead: Delivery[];
  signedOut: Bot[];
  onAnswered(approvalId: string): void;
}) {
  const t = useT().home;
  const [failedOpen, setFailedOpen] = useState(false);
  const selectBot = useApp((state) => state.selectBot);
  const entries: Entry[] = [
    ...requests.map((card) => ({ kind: "request" as const, at: card.createdAt, card })),
    ...questions.map((question) => ({
      kind: "question" as const,
      at: question.createdAt,
      question,
    })),
  ].sort((a, b) => a.at - b.at);

  if (entries.length === 0 && dead.length === 0 && signedOut.length === 0) {
    return (
      <EmptyState framed color={AVATAR_PALETTE[0]} title={t.empty.title} body={t.empty.body} />
    );
  }
  return (
    <>
      <ul className="flex flex-col gap-6">
        {entries.map((entry) =>
          entry.kind === "request" ? (
            <RequestEntry
              key={entry.card.approvalId}
              card={entry.card}
              onDone={() => onAnswered(entry.card.approvalId)}
            />
          ) : (
            <QuestionEntry key={entry.question.id} question={entry.question} />
          ),
        )}
      </ul>
      {(dead.length > 0 || signedOut.length > 0) && (
        <ul className="mt-4 flex flex-col overflow-hidden rounded-xl border border-line bg-panel">
          {dead.length > 0 && (
            <Line
              icon={<CircleX aria-hidden size={15} className="text-danger" />}
              text={t.failed.line(dead.length)}
            >
              <Button size="sm" onClick={() => setFailedOpen(true)}>
                {t.failed.show}
              </Button>
            </Line>
          )}
          {signedOut.map((bot) => (
            <Line
              key={bot.id}
              icon={<KeyRound aria-hidden size={15} className="text-danger" />}
              text={t.signedOut.line(bot.name)}
            >
              <Button size="sm" onClick={() => selectBot(bot.id)}>
                {t.signedOut.open}
              </Button>
            </Line>
          ))}
        </ul>
      )}
      {failedOpen && <FailedDialog dead={dead} onClose={() => setFailedOpen(false)} />}
    </>
  );
}

function Line({ icon, text, children }: { icon: ReactNode; text: string; children: ReactNode }) {
  return (
    <li className="flex items-center gap-2.5 border-line border-b px-3.5 py-2.5 text-sm last:border-b-0">
      {icon}
      <span className="min-w-0 flex-1">{text}</span>
      {children}
    </li>
  );
}
