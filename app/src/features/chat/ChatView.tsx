// A bot's chat (spec 8 and 15): its items oldest first, the reply being
// written, and the composer. Files dropped anywhere on it are attached.

import { LoaderCircle, Paperclip } from "lucide-react";
import { type DragEvent, type ReactNode, useLayoutEffect, useRef, useState } from "react";
import { day } from "../../lib/format";
import type { Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { BotAvatar } from "../bots/BotAvatar";
import { BotRun, type Live } from "./BotRun";
import { ChatComposer } from "./ChatComposer";
import { InboundRow } from "./InboundRow";
import { NoticeRow } from "./NoticeRow";
import { chatRows } from "./rows";
import { useChat } from "./useChat";
import { useFiles } from "./useFiles";

/** Distance from the bottom that still counts as "reading the latest". */
const STICKY_PX = 80;

const hasFiles = (event: DragEvent) => event.dataTransfer.types.includes("Files");

export function ChatView({ bot, stopped }: { bot: Bot; stopped: boolean }) {
  const chat = useChat(bot.id);
  const files = useFiles();
  const scroller = useRef<HTMLDivElement>(null);
  const atEnd = useRef(true);
  /** Distance from the bottom to keep while older items load above. */
  const anchor = useRef<number | null>(null);
  const [dragging, setDragging] = useState(false);
  const last = chat.items.at(-1);
  /** Changes whenever the end of the chat does. */
  const end = `${chat.items.length}/${last?.id}/${last?.updatedAt}/${chat.draft.length}`;
  const seenTail = useRef("");

  useLayoutEffect(() => {
    const element = scroller.current;
    if (!element || end === seenTail.current) {
      return;
    }
    seenTail.current = end;
    if (anchor.current !== null) {
      element.scrollTop = element.scrollHeight - anchor.current;
      anchor.current = null;
    } else if (atEnd.current) {
      element.scrollTop = element.scrollHeight;
    }
  }, [end]);

  const loadOlder = () => {
    const element = scroller.current;
    if (element) {
      anchor.current = element.scrollHeight - element.scrollTop;
    }
    void chat.loadOlder();
  };

  const working = bot.state === "busy";
  const live: Live | undefined = chat.draft || working ? { draft: chat.draft, working } : undefined;
  const rows = chatRows(chat.items);
  const tail = rows.at(-1);
  const openRun = tail?.kind === "run" && tail.items.at(-1)?.body.kind !== "turn";

  const list: ReactNode[] = rows.map((row, index) => {
    if (row.kind === "day") {
      return (
        <li key={row.key} className="flex items-center gap-3 text-muted text-xs">
          <span className="h-px flex-1 bg-line" />
          {day(row.at)}
          <span className="h-px flex-1 bg-line" />
        </li>
      );
    }
    if (row.kind === "inbound") {
      return <InboundRow key={row.key} message={row.message} bot={bot} />;
    }
    if (row.kind === "notice") {
      return <NoticeRow key={row.key} notice={row.notice} />;
    }
    const isLast = index === rows.length - 1;
    return (
      <BotRun
        key={row.key}
        items={row.items}
        bot={bot}
        live={isLast && openRun ? live : undefined}
      />
    );
  });
  if (live && !openRun) {
    list.push(<BotRun key="live" items={[]} bot={bot} live={live} />);
  }

  return (
    <section
      aria-label={`Chat with ${bot.name}`}
      className="relative flex min-h-0 flex-1 flex-col"
      onDragOver={(event) => {
        if (hasFiles(event)) {
          event.preventDefault();
          setDragging(true);
        }
      }}
      onDragLeave={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Node | null)) {
          setDragging(false);
        }
      }}
      onDrop={(event) => {
        if (hasFiles(event)) {
          event.preventDefault();
          setDragging(false);
          void files.add([...event.dataTransfer.files]);
        }
      }}
    >
      <div
        ref={scroller}
        className="min-h-0 flex-1 overflow-y-auto"
        onScroll={(event) => {
          const element = event.currentTarget;
          atEnd.current =
            element.scrollHeight - element.scrollTop - element.clientHeight < STICKY_PX;
        }}
      >
        <div className="mx-auto flex max-w-5xl flex-col px-5 pt-4 pb-2">
          {chat.error && (
            <Callout tone="danger" title="Could not load the chat">
              {chat.error}
            </Callout>
          )}
          {chat.items.length > 0 && !chat.complete && (
            <div className="flex justify-center pb-3">
              <Button size="sm" disabled={chat.loading} onClick={loadOlder}>
                Load earlier messages
              </Button>
            </div>
          )}
          {chat.items.length === 0 && chat.loading && (
            <p role="status" className="flex items-center gap-2 py-6 text-muted text-sm">
              <LoaderCircle aria-hidden size={14} className="animate-spin" />
              Loading the chat…
            </p>
          )}
          {chat.items.length === 0 && !chat.loading && !chat.error && !live && (
            <div className="flex flex-col items-center gap-3 py-16 text-center">
              <BotAvatar color={bot.color} size={56} />
              <p className="font-semibold text-base">Start a conversation with {bot.name}</p>
              <p className="max-w-md text-muted text-sm">
                {bot.role || "Ask for anything its folder and tools can do."} You can attach files
                and images too.
              </p>
            </div>
          )}
          <ol aria-label="Messages" className="flex flex-col gap-5">
            {list}
          </ol>
        </div>
      </div>
      <div className="mx-auto w-full max-w-5xl">
        <ChatComposer
          bot={bot}
          files={files}
          stopped={stopped}
          onSent={() => {
            atEnd.current = true;
          }}
        />
      </div>
      {dragging && (
        <div className="pointer-events-none absolute inset-3 grid place-items-center rounded-lg border-2 border-accent border-dashed bg-canvas/85">
          <p className="flex items-center gap-2 font-medium">
            <Paperclip aria-hidden size={16} />
            Drop to attach to your message
          </p>
        </div>
      )}
    </section>
  );
}
