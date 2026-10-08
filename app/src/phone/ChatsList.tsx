// The list of conversations (spec 28.12): each bot with its last activity,
// "working…" while it is busy and a dot when it wrote since the owner looked.

import { useT } from "../i18n";
import type { ChatLine } from "../lib/protocol.gen";
import { BotDot } from "./parts";

export function isUnread(line: ChatLine, seen: Record<string, number>): boolean {
  return line.lastReplyAt !== undefined && line.lastReplyAt > (seen[line.botId] ?? 0);
}

export function ChatsList({
  chats,
  loaded,
  seen,
  open,
}: {
  chats: ChatLine[];
  loaded: boolean;
  seen: Record<string, number>;
  open(botId: string): void;
}) {
  const t = useT().phone;
  if (chats.length === 0) {
    return (
      <div className="flex flex-col items-center gap-2 py-16 text-center">
        {loaded ? (
          <>
            <p className="font-medium text-base">{t.chats.empty}</p>
            <p className="text-muted text-sm">{t.chats.emptyBody}</p>
          </>
        ) : (
          <p className="text-muted text-sm">{t.chats.loading}</p>
        )}
      </div>
    );
  }
  return (
    <ul className="flex flex-col gap-2">
      {chats.map((line) => (
        <li key={line.botId}>
          <ChatRow line={line} unread={isUnread(line, seen)} open={() => open(line.botId)} />
        </li>
      ))}
    </ul>
  );
}

function ChatRow({ line, unread, open }: { line: ChatLine; unread: boolean; open(): void }) {
  const t = useT().phone;
  const busy = line.state === "busy" || line.state === "needs_approval";
  const kind = line.last ? t.chats.kind[line.last.kind] : "";
  const last = line.last ? `${kind ? `${kind}: ` : ""}${line.last.text}` : "";
  return (
    <button
      type="button"
      onClick={open}
      className="flex min-h-16 w-full items-center gap-3 rounded-2xl border border-line bg-panel p-3 text-left active:scale-[0.99]"
    >
      <BotDot name={line.name} color={line.color} />
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="flex items-center gap-2">
          <span className="truncate font-medium">{line.name}</span>
          <span className="truncate text-muted text-xs">{t.chats.inCrew(line.crew)}</span>
        </span>
        <span className="truncate text-ink-soft text-sm">{busy ? t.chats.working : last}</span>
      </span>
      {unread && (
        <span
          role="img"
          aria-label={t.tabs.newReply}
          className="h-3 w-3 shrink-0 rounded-full bg-work"
        />
      )}
    </button>
  );
}
