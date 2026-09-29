// A message to the bot, in its chat: the owner's on the right in a bubble,
// another bot's or Botloft's on the left under its name (spec 15.3).

import { ListTodo, Reply } from "lucide-react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { Bot, Message } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { BotAvatar } from "../bots/BotAvatar";
import { DeliveryStatus } from "../messages/DeliveryStatus";
import { AttachmentList } from "./AttachmentList";

function Time({ at }: { at: number }) {
  return (
    <time className="text-muted text-xs" dateTime={new Date(at).toISOString()}>
      {when(at)}
    </time>
  );
}

function OwnerMessage({ message, bot }: { message: Message; bot: Bot }) {
  const delivery = useApp((state) => state.deliveries[message.id]);
  return (
    <li className="flex flex-col items-end gap-1.5 pl-16">
      <AttachmentList attachments={message.attachments} workspace={bot.workspace} align="end" />
      {message.body && (
        <p
          className="max-w-[42rem] whitespace-pre-wrap break-words rounded-lg rounded-br-sm bg-sunken px-3.5 py-2 leading-relaxed"
          data-selectable
        >
          {message.body}
        </p>
      )}
      <div className="flex items-center gap-2">
        {delivery && <DeliveryStatus delivery={delivery} />}
        <Time at={message.createdAt} />
      </div>
    </li>
  );
}

function TaskTag({ message }: { message: Message }) {
  const t = useT();
  if (message.kind !== "task" && message.kind !== "result") {
    return null;
  }
  const Icon = message.kind === "task" ? ListTodo : Reply;
  return (
    <span className="inline-flex items-center gap-1 rounded-[3px] border border-line px-1.5 text-muted text-xs">
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
  const name = system ? "Botloft" : (sender?.name ?? t.chat.inbound.archivedBot);
  return (
    <li className="flex gap-3 pr-16">
      <BotAvatar
        color={system ? "#ffffff" : (sender?.color ?? "#6f6f69")}
        size={28}
        framed={system}
      />
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-x-2 text-sm">
          <span className="font-semibold">{name}</span>
          {sender && <span className="font-mono text-muted text-xs">@{sender.handle}</span>}
          <TaskTag message={message} />
          <Time at={message.createdAt} />
        </div>
        <p
          className={`mt-1 max-w-[42rem] whitespace-pre-wrap break-words rounded-lg rounded-tl-sm border border-line bg-panel px-3.5 py-2 leading-relaxed ${system ? "text-ink-soft" : ""}`}
          data-selectable
        >
          {message.body}
        </p>
        {delivery && delivery.state !== "sent" && (
          <div className="mt-1">
            <DeliveryStatus delivery={delivery} />
          </div>
        )}
      </div>
    </li>
  );
}

export function InboundRow({ message, bot }: { message: Message; bot: Bot }) {
  return message.fromKind === "owner" ? (
    <OwnerMessage message={message} bot={bot} />
  ) : (
    <OtherMessage message={message} />
  );
}
