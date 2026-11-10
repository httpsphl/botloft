// Replying to something in the bot's chat (spec 9.3): a button that shows
// beside what can be quoted (the bot's replies, the messages it got), the
// quote over the composer, and the quote over the owner's bubble after.

import { CornerUpLeft, X } from "lucide-react";
import { createContext, type ReactNode, useContext } from "react";
import { useT } from "../../i18n";
import { clock } from "../../lib/format";
import { plainText } from "../../lib/plainText";
import type { ChatItemId, MessageReply } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { asks } from "./asks";
import { ReactButton, ReactionMark } from "./Reactions";

/** What the owner is replying to, while they write. */
export interface ReplyTarget {
  itemId: ChatItemId;
  /** Whose words: the bot's name, another bot's, or Botloft. */
  who: string;
  text: string;
}

/** Starts a reply; absent where the chat cannot be written to. */
export const StartReply = createContext<((target: ReplyTarget) => void) | null>(null);

/**
 * Wraps a balloon that can be replied to, with what shows beside it on
 * hover: its time, reply and, for the bot's own replies, react (spec 8.9).
 * The chat leaves room on the right for them.
 */
export function Repliable({
  target,
  at,
  react = false,
  children,
}: {
  target: ReplyTarget;
  /** When it was said. */
  at: number;
  /** The bot's own reply: the owner can react to it too. */
  react?: boolean;
  children: ReactNode;
}) {
  const t = useT().chat.reply;
  const start = useContext(StartReply);
  if (!start) {
    return children;
  }
  // A 👍 or 👎 answers a reply that ends with a question (spec 8.9).
  const asked = react && asks(target.text);
  return (
    <div className="group/reply relative w-fit max-w-full">
      {children}
      {react && <ReactionMark itemId={target.itemId} bot={target.who} asked={asked} />}
      <div className="absolute top-0 left-full flex h-10 items-center gap-0.5 pl-1.5 opacity-0 transition-opacity focus-within:opacity-100 group-hover/reply:opacity-100">
        <time
          className="whitespace-nowrap px-0.5 text-muted text-xs tabular-nums"
          dateTime={new Date(at).toISOString()}
        >
          {clock(at)}
        </time>
        {react && <ReactButton itemId={target.itemId} bot={target.who} asked={asked} />}
        <button
          type="button"
          title={t.action}
          aria-label={t.to(target.who)}
          onClick={() => start(target)}
          className="grid size-7 place-items-center rounded-full text-muted transition-colors hover:bg-sunken hover:text-ink"
        >
          <CornerUpLeft aria-hidden size={15} />
        </button>
      </div>
    </div>
  );
}

/** Over the composer: what the next message replies to, with a way out. */
export function ReplyBar({ target, onCancel }: { target: ReplyTarget; onCancel(): void }) {
  const t = useT().chat.reply;
  return (
    <div className="flex animate-rise items-start gap-2 border-line border-b px-3.5 py-2">
      <CornerUpLeft aria-hidden size={14} className="mt-0.5 shrink-0 text-accent" />
      <div className="min-w-0 flex-1 text-xs">
        <p className="font-medium text-ink-soft">{t.replyingTo(target.who)}</p>
        <p className="truncate text-muted">{plainText(target.text)}</p>
      </div>
      <button
        type="button"
        title={t.cancel}
        aria-label={t.cancel}
        onClick={onCancel}
        className="grid size-6 shrink-0 place-items-center rounded-lg text-muted transition-colors hover:bg-sunken hover:text-ink"
      >
        <X aria-hidden size={13} />
      </button>
    </div>
  );
}

/** Over the owner's bubble: what it replied to; a click shows it in the chat. */
export function ReplyQuote({ botId, reply }: { botId: string; reply: MessageReply }) {
  const t = useT().chat.reply;
  const openAt = useApp((state) => state.openAt);
  return (
    <button
      type="button"
      title={t.jump}
      onClick={() => openAt(botId, reply.itemId)}
      className="flex max-w-[36rem] items-start gap-1.5 rounded-xl border-accent/60 border-l-2 bg-bubble px-3 py-1.5 text-left text-muted text-xs transition-colors hover:text-ink"
    >
      <CornerUpLeft aria-hidden size={12} className="mt-0.5 shrink-0" />
      <span className="line-clamp-2">{plainText(reply.text)}</span>
    </button>
  );
}
