// Replying to something in the bot's chat (spec 9.3): a button that shows
// on what can be quoted (the bot's replies, the messages it got), the
// quote over the composer, and the quote over the owner's bubble after.

import { CornerUpLeft, X } from "lucide-react";
import { createContext, type ReactNode, useContext } from "react";
import { useT } from "../../i18n";
import { plainText } from "../../lib/plainText";
import type { ChatItemId, MessageReply } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
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
 * Wraps what can be replied to, with the buttons that show on hover: reply,
 * and for the bot's own replies, react (spec 8.9).
 */
export function Repliable({
  target,
  react = false,
  className = "",
  children,
}: {
  target: ReplyTarget;
  /** The bot's own reply: the owner can react to it too. */
  react?: boolean;
  /** A bubble narrows it to its own width, for the buttons to sit on its corner. */
  className?: string;
  children: ReactNode;
}) {
  const t = useT().chat.reply;
  const start = useContext(StartReply);
  if (!start) {
    return children;
  }
  return (
    <div className={`group/reply relative ${className}`}>
      {children}
      {react && <ReactionMark itemId={target.itemId} bot={target.who} />}
      <div className="absolute -top-2 right-0 flex rounded-lg border border-line bg-panel opacity-0 shadow-sm transition-opacity focus-within:opacity-100 group-hover/reply:opacity-100">
        {react && <ReactButton itemId={target.itemId} bot={target.who} />}
        <button
          type="button"
          title={t.action}
          aria-label={t.to(target.who)}
          onClick={() => start(target)}
          className="grid size-7 place-items-center rounded-lg text-muted transition-colors hover:text-ink"
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
      className="flex max-w-[36rem] items-start gap-1.5 rounded-xl border-accent/60 border-l-2 bg-sunken/60 px-3 py-1.5 text-left text-muted text-xs transition-colors hover:text-ink"
    >
      <CornerUpLeft aria-hidden size={12} className="mt-0.5 shrink-0" />
      <span className="line-clamp-2">{plainText(reply.text)}</span>
    </button>
  );
}
