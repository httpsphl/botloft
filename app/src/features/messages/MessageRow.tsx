import { ArrowRight, ListTodo, Reply, User } from "lucide-react";
import { useT } from "../../i18n";
import { fromNow, when } from "../../lib/format";
import type { Bot, BotId, Message, Task } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { BotAvatar } from "../bots/BotAvatar";
import { DeliveryStatus } from "./DeliveryStatus";

function BotName({ bot }: { bot: Bot | undefined }) {
  const t = useT();
  if (!bot) {
    return <span className="text-muted">{t.messages.row.archivedBot}</span>;
  }
  return (
    <span className="inline-flex items-center gap-1.5 font-medium">
      <BotAvatar color={bot.color} size={14} />
      {bot.name}
    </span>
  );
}

function Sender({ message, bots }: { message: Message; bots: Record<BotId, Bot> }) {
  const t = useT();
  switch (message.fromKind) {
    case "owner":
      return (
        <span className="inline-flex items-center gap-1.5 font-medium">
          <User aria-hidden size={14} className="text-muted" />
          {t.messages.row.you}
        </span>
      );
    case "system":
      return (
        <span className="inline-flex items-center gap-1.5 font-medium">
          <BotAvatar color="#ffffff" size={14} />
          {t.messages.row.system}
        </span>
      );
    case "bot":
      return <BotName bot={message.fromBotId ? bots[message.fromBotId] : undefined} />;
  }
}

/** What a task or result message is about, from the task's current state. */
function TaskTag({ message, task }: { message: Message; task: Task | undefined }) {
  const text = useT().messages.row;
  if (message.kind !== "task" && message.kind !== "result") {
    return null;
  }
  const Icon = message.kind === "task" ? ListTodo : Reply;
  let detail = "";
  if (task) {
    detail =
      task.status === "open"
        ? `${text.taskStatus.open} · ${text.due(fromNow(task.deadlineAt))}`
        : text.taskStatus[task.status];
  }
  return (
    <span className="inline-flex items-center gap-1 border border-line px-1.5 text-muted text-xs">
      <Icon aria-hidden size={11} />
      {message.kind === "task" ? text.task : text.result}
      {detail && <span>· {detail}</span>}
    </span>
  );
}

export function MessageRow({ message }: { message: Message }) {
  const t = useT();
  const bots = useApp((state) => state.bots);
  const task = useApp((state) => (message.taskId ? state.tasks[message.taskId] : undefined));
  const delivery = useApp((state) => state.deliveries[message.id]);
  return (
    <li className="border-line border-b px-5 py-3 last:border-b-0">
      <div className="flex items-center gap-2 text-sm">
        <Sender message={message} bots={bots} />
        <ArrowRight aria-label={t.messages.row.to} size={12} className="text-muted" />
        <BotName bot={bots[message.toBotId]} />
        <TaskTag message={message} task={task} />
        <time
          className="ml-auto shrink-0 text-muted text-xs"
          dateTime={new Date(message.createdAt).toISOString()}
        >
          {when(message.createdAt)}
        </time>
      </div>
      <p
        className={`mt-1 whitespace-pre-wrap break-words text-sm leading-relaxed ${message.kind === "system" ? "text-muted" : "text-ink"}`}
        data-selectable
      >
        {message.body}
      </p>
      {delivery && (
        <div className="mt-1">
          <DeliveryStatus delivery={delivery} />
        </div>
      )}
    </li>
  );
}
