// The things in an open conversation (spec 28.12): the owner's words on the
// right, the bot's on the left, tool calls as one quiet line, and a request
// or a question that waits as its card, with the same buttons as the inbox.

import { useT } from "../i18n";
import type { PhoneItem } from "../lib/protocol.gen";
import { Callout } from "../ui/Callout";
import { ApprovalView } from "./ApprovalView";
import type { PhoneApi, PhoneState } from "./client";
import { PhoneMarkdown } from "./parts";
import { QuestionView } from "./QuestionView";
import type { Pending } from "./talk";

function Cut() {
  return <p className="text-muted text-xs">{useT().phone.chats.cut}</p>;
}

function Mine({ text, cut, note }: { text: string; cut?: boolean; note?: string }) {
  return (
    <div className="flex max-w-[85%] flex-col items-end gap-1 self-end">
      <p className="whitespace-pre-wrap break-words rounded-2xl bg-ink px-3 py-2 text-base text-canvas">
        {text}
      </p>
      {cut && <Cut />}
      {note && <span className="text-muted text-xs">{note}</span>}
    </div>
  );
}

export function ChatItem({
  item,
  api,
  state,
}: {
  item: PhoneItem;
  api: PhoneApi;
  state: PhoneState;
}) {
  const t = useT().phone;
  switch (item.kind) {
    case "you":
      return <Mine text={item.text} cut={item.cut} />;
    case "bot_message":
      return (
        <div className="flex max-w-[85%] flex-col gap-1 self-start rounded-2xl border border-line bg-sunken px-3 py-2">
          <span className="text-muted text-xs">{t.chats.from(item.from)}</span>
          <p className="whitespace-pre-wrap break-words text-base">{item.text}</p>
          {item.cut && <Cut />}
        </div>
      );
    case "reply":
      return (
        <div className="flex max-w-full flex-col gap-1 self-start rounded-2xl border border-line bg-panel px-3 py-2">
          <PhoneMarkdown>{item.text}</PhoneMarkdown>
          {item.cut && <Cut />}
        </div>
      );
    case "tool":
      return <p className="break-words text-muted text-xs">{t.chats.tool(item.summary)}</p>;
    case "approval": {
      const card = state.approvals.find((one) => one.approvalId === item.approvalId);
      if (card) {
        return (
          <ApprovalView
            card={card}
            sending={state.sending.includes(card.approvalId)}
            answer={(allow, note) => api.answerApproval(card.approvalId, allow, note)}
          />
        );
      }
      return (
        <p className="break-words text-muted text-sm">
          {t.chats.approvalLine(item.summary)}
          {item.status !== "pending" && ` — ${t.approval.ended[item.status]}`}
        </p>
      );
    }
    case "question": {
      const card = state.questions.find((one) => one.questionId === item.questionId);
      if (card) {
        return (
          <QuestionView
            card={card}
            sending={state.sending.includes(card.questionId)}
            answer={(text) => api.answerQuestion(card.questionId, text)}
            dismiss={() => api.dismissQuestion(card.questionId)}
          />
        );
      }
      const ended = (t.question.ended as Record<string, string>)[item.status];
      return (
        <p className="break-words text-muted text-sm">
          {t.chats.questionLine(item.text)}
          {ended && ` — ${ended}`}
        </p>
      );
    }
    case "failed":
      return (
        <p role="alert" className="break-words text-danger text-sm">
          {t.chats.failedTurn}
          {item.error ? `: ${item.error}` : ""}
        </p>
      );
    case "notice":
      return <Callout tone={item.level === "info" ? "info" : "warn"} title={item.text} />;
    default:
      return null;
  }
}

/** What the owner wrote that the computer has not shown back yet. */
export function PendingItem({ pending }: { pending: Pending }) {
  const c = useT().phone.chats;
  const reason = c.pending.reasons[pending.reason ?? "failed"] ?? c.pending.reasons.failed;
  const note =
    pending.status === "sending"
      ? c.pending.sending
      : pending.status === "sent"
        ? c.pending.sent
        : `${c.pending.notSent} ${reason}`;
  return (
    <div className={pending.status === "failed" ? "self-end text-danger" : "self-end"}>
      <Mine text={pending.text} note={note} />
    </div>
  );
}
