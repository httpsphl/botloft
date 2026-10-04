// A message to the bot, in its chat: the owner's on the right in a bubble,
// another bot's or Botloft's on the left under its name (spec 15.3).

import { AlarmClock, ChevronRight, ListTodo, Reply } from "lucide-react";
import { memo, useState } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { Bot, Message } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { useArrival } from "../../ui/motion";
import { BOTLOFT_COLOR, BotAvatar } from "../bots/BotAvatar";
import { DeliveryStatus } from "../messages/DeliveryStatus";
import { AttachmentList } from "./AttachmentList";
import { Mentions } from "./mentions";

function Time({ at }: { at: number }) {
  return (
    <time className="text-muted text-xs" dateTime={new Date(at).toISOString()}>
      {when(at)}
    </time>
  );
}

function OwnerMessage({ message, bot }: { message: Message; bot: Bot }) {
  const delivery = useApp((state) => state.deliveries[message.id]);
  const arrival = useArrival(message.createdAt);
  return (
    <li className={`flex flex-col items-end gap-1.5 pl-12 ${arrival}`}>
      <AttachmentList attachments={message.attachments} workspace={bot.workspace} align="end" />
      {message.body && (
        <p
          className="max-w-[36rem] whitespace-pre-wrap break-words rounded-2xl rounded-br-md bg-sunken px-4 py-2.5 leading-relaxed"
          data-selectable
        >
          <Mentions text={message.body} />
        </p>
      )}
      <div className="flex items-center gap-2">
        {delivery && <DeliveryStatus delivery={delivery} />}
        <Time at={message.createdAt} />
      </div>
    </li>
  );
}

/** A routine's tag, which opens what the routine asks: folded, it is only the name. */
function RoutineTag({
  message,
  open,
  onToggle,
}: {
  message: Message;
  open: boolean;
  onToggle: () => void;
}) {
  const t = useT();
  const routine = useApp((state) =>
    message.routineId ? state.routines[message.routineId] : undefined,
  );
  return (
    <button
      type="button"
      aria-expanded={open}
      title={t.chat.inbound.routine}
      onClick={onToggle}
      className="inline-flex items-center gap-1 rounded-lg border border-accent/40 px-1.5 text-accent-text text-xs transition-colors hover:bg-accent/10"
    >
      <AlarmClock aria-hidden size={11} />
      {t.routines.tag(routine?.name ?? t.routines.tab)}
      <ChevronRight
        aria-hidden
        size={11}
        className={`transition-transform ${open ? "rotate-90" : ""}`}
      />
    </button>
  );
}

function TaskTag({ message }: { message: Message }) {
  const t = useT();
  if (message.kind !== "task" && message.kind !== "result") {
    return null;
  }
  const Icon = message.kind === "task" ? ListTodo : Reply;
  return (
    <span className="inline-flex items-center gap-1 rounded-lg border border-line px-1.5 text-muted text-xs">
      <Icon aria-hidden size={11} />
      {message.kind === "task" ? t.chat.inbound.task : t.chat.inbound.result}
    </span>
  );
}

function OtherMessage({ message }: { message: Message }) {
  const t = useT();
  const sender = useApp((state) => (message.fromBotId ? state.bots[message.fromBotId] : undefined));
  const delivery = useApp((state) => state.deliveries[message.id]);
  const system = message.fromKind === "system";
  const name = system ? "Botloft" : (sender?.name ?? t.chat.inbound.goneBot);
  const arrival = useArrival(message.createdAt);
  const routine = message.kind === "routine";
  const [open, setOpen] = useState(false);
  return (
    <li className={`flex gap-3 pr-12 ${arrival}`}>
      <BotAvatar
        color={system ? BOTLOFT_COLOR : (sender?.color ?? "#6f6f69")}
        size={28}
        framed={system}
      />
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-x-2 text-sm">
          <span className="font-semibold">{name}</span>
          {sender && <span className="font-mono text-muted text-xs">@{sender.handle}</span>}
          {routine ? (
            <RoutineTag message={message} open={open} onToggle={() => setOpen(!open)} />
          ) : (
            <TaskTag message={message} />
          )}
          <Time at={message.createdAt} />
        </div>
        {(!routine || open) && (
          <p
            className={`mt-1 max-w-[36rem] ${routine ? "animate-rise" : ""} whitespace-pre-wrap break-words rounded-2xl rounded-tl-md border border-line bg-panel px-4 py-2.5 leading-relaxed ${system ? "text-ink-soft" : ""}`}
            data-selectable
          >
            <Mentions text={message.body} />
          </p>
        )}
        {delivery && delivery.state !== "sent" && (
          <div className="mt-1">
            <DeliveryStatus delivery={delivery} />
          </div>
        )}
      </div>
    </li>
  );
}

export const InboundRow = memo(function InboundRow({
  message,
  bot,
}: {
  message: Message;
  bot: Bot;
}) {
  return message.fromKind === "owner" ? (
    <OwnerMessage message={message} bot={bot} />
  ) : (
    <OtherMessage message={message} />
  );
});
