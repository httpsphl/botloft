// A message to the bot, in its chat: the owner's on the right in a dark
// balloon, another bot's or Botloft's on the left under its name (spec 15.3).

import { AlarmClock, ChevronRight, ListTodo, Reply } from "lucide-react";
import { memo, useState } from "react";
import { useT } from "../../i18n";
import { clock } from "../../lib/format";
import type { Bot, Delivery, Message } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { useArrival } from "../../ui/motion";
import { BOTLOFT_COLOR, BotAvatar } from "../bots/BotAvatar";
import { DeliveryStatus } from "../messages/DeliveryStatus";
import { AttachmentList } from "./AttachmentList";
import { ChatText } from "./ChatText";
import { Repliable, ReplyQuote } from "./Replying";

/** Read by the bot: the line under it no longer says anything new. */
const settled = (delivery: Delivery) => delivery.state === "sent" && delivery.readAt !== null;

function OwnerMessage({ message, bot, last }: { message: Message; bot: Bot; last: boolean }) {
  const delivery = useApp((state) => state.deliveries[message.id]);
  const arrival = useArrival(message.createdAt);
  return (
    <li className={`flex flex-col items-end gap-1 pl-14 lg:pl-32 ${arrival}`}>
      {message.replyTo && <ReplyQuote botId={bot.id} reply={message.replyTo} />}
      <AttachmentList attachments={message.attachments} workspace={bot.workspace} align="end" />
      {message.body && (
        <div className="group/own relative max-w-full">
          {/* Its time shows beside it on hover, as the bot's balloons do. */}
          <time
            className="absolute top-0 right-full flex h-10 items-center whitespace-nowrap pr-2.5 text-muted text-xs tabular-nums opacity-0 transition-opacity group-hover/own:opacity-100"
            dateTime={new Date(message.createdAt).toISOString()}
          >
            {clock(message.createdAt)}
          </time>
          <p
            className="chat-own max-w-[36rem] whitespace-pre-wrap break-words rounded-bubble bg-bubble-own px-4 py-2.5 text-bubble-own-ink leading-relaxed [&_a]:text-bubble-own-link"
            data-selectable
          >
            <ChatText text={message.body} />
          </p>
        </div>
      )}
      {/* Where it is, under the newest one and under any still on its way. */}
      {delivery && (last || !settled(delivery)) && (
        <div className="px-1">
          <DeliveryStatus delivery={delivery} />
        </div>
      )}
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
    <span className="inline-flex items-center gap-1 rounded-full bg-sunken px-2 text-muted text-xs">
      <Icon aria-hidden size={11} />
      {message.kind === "task" ? t.chat.inbound.task : t.chat.inbound.result}
    </span>
  );
}

function OtherMessage({ message, itemId }: { message: Message; itemId: string }) {
  const t = useT();
  const sender = useApp((state) => (message.fromBotId ? state.bots[message.fromBotId] : undefined));
  const delivery = useApp((state) => state.deliveries[message.id]);
  const system = message.fromKind === "system";
  const name = system ? "Botloft" : (sender?.name ?? t.chat.inbound.goneBot);
  const arrival = useArrival(message.createdAt);
  const routine = message.kind === "routine";
  const [open, setOpen] = useState(false);
  return (
    <li className={`flex pr-14 lg:pr-32 ${arrival}`}>
      <div className="flex min-w-0 flex-1 flex-col items-start gap-1">
        <div className="flex flex-wrap items-center gap-x-1.5 gap-y-1 pl-1 text-xs">
          <BotAvatar
            color={system ? BOTLOFT_COLOR : (sender?.color ?? "#6f6f69")}
            size={18}
            framed={system}
          />
          <span className="font-semibold text-ink-soft">{name}</span>
          {routine ? (
            <RoutineTag message={message} open={open} onToggle={() => setOpen(!open)} />
          ) : (
            <TaskTag message={message} />
          )}
        </div>
        {(!routine || open) && (
          <Repliable target={{ itemId, who: name, text: message.body }} at={message.createdAt}>
            <p
              className={`max-w-[36rem] ${routine ? "animate-rise" : ""} whitespace-pre-wrap break-words rounded-bubble border border-line bg-panel px-4 py-2.5 leading-relaxed ${system ? "text-ink-soft" : ""}`}
              data-selectable
            >
              <ChatText text={message.body} />
            </p>
          </Repliable>
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
  itemId,
  last = false,
}: {
  message: Message;
  bot: Bot;
  /** The chat item that shows it, to reply to. */
  itemId: string;
  /** The owner's newest message in the chat, which always says where it is. */
  last?: boolean;
}) {
  return message.fromKind === "owner" ? (
    <OwnerMessage message={message} bot={bot} last={last} />
  ) : (
    <OtherMessage message={message} itemId={itemId} />
  );
});
