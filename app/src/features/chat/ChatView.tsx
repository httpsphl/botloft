// A bot's chat (spec 8 and 15): its items oldest first, the reply being
// written, and the composer. Files dropped anywhere on it are attached.

import { LoaderCircle, Paperclip } from "lucide-react";
import {
  type DragEvent,
  memo,
  type ReactNode,
  useCallback,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useT } from "../../i18n";
import { day } from "../../lib/format";
import type { Bot } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { SeenSince } from "../../ui/motion";
import { BotAvatar, moodOf } from "../bots/BotAvatar";
import { BotRun, type Live } from "./BotRun";
import { ChatComposer } from "./ChatComposer";
import { InboundRow } from "./InboundRow";
import { NoticeRow } from "./NoticeRow";
import { type ReplyTarget, StartReply } from "./Replying";
import { chatRows, rowHas } from "./rows";
import { useChat } from "./useChat";
import { useFiles } from "./useFiles";
import { useSharedSound } from "./useSharedSound";

/** How long a search result stays marked (`search.css`). */
const FOCUS_MS = 2400;

/** Distance from the bottom that still counts as "reading the latest". */
const STICKY_PX = 80;

const hasFiles = (event: DragEvent) => event.dataTransfer.types.includes("Files");

/** Skips the renders of the view around it; the text being written re-renders only the last run. */
export const ChatView = memo(function ChatView({ bot, stopped }: { bot: Bot; stopped: boolean }) {
  const t = useT();
  const focus = useApp((state) => (state.focus?.botId === bot.id ? state.focus.itemId : null));
  const chat = useChat(bot.id, focus);
  const files = useFiles();
  const scroller = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLOListElement>(null);
  /** The search result already shown, so it is not scrolled to again. */
  const shownFocus = useRef<string | null>(null);
  const atEnd = useRef(true);
  /** Distance from the bottom to keep while older items load above. */
  const anchor = useRef<number | null>(null);
  const [dragging, setDragging] = useState(false);
  /** What the owner's next message replies to (spec 9.3). */
  const [replying, setReplying] = useState<ReplyTarget | null>(null);
  const cancelReply = useCallback(() => setReplying(null), []);
  // What was there when the owner opened this chat stays still; what
  // arrives while they look animates in.
  // biome-ignore lint/correctness/useExhaustiveDependencies: a new bot is a new chat
  const openedAt = useMemo(() => Date.now(), [bot.id]);
  useSharedSound(chat.items, openedAt);
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

  const onSent = useCallback(() => {
    atEnd.current = true;
  }, []);

  const loadOlder = () => {
    const element = scroller.current;
    if (element) {
      anchor.current = element.scrollHeight - element.scrollTop;
    }
    void chat.loadOlder();
  };

  const working = bot.state === "busy";
  const live: Live | undefined = chat.draft || working ? { draft: chat.draft, working } : undefined;
  const rows = useMemo(() => chatRows(chat.items), [chat.items]);
  const tail = rows.at(-1);
  const openRun = tail?.kind === "run" && tail.items.at(-1)?.body.kind !== "turn";

  const list: ReactNode[] = rows.map((row, index) => {
    if (row.kind === "day") {
      return (
        <li key={row.key} className="flex animate-fade justify-center">
          <span className="rounded-full bg-sunken px-3 py-1 font-medium text-muted text-xs">
            {day(row.at)}
          </span>
        </li>
      );
    }
    if (row.kind === "inbound") {
      return <InboundRow key={row.key} message={row.message} bot={bot} itemId={row.item.id} />;
    }
    if (row.kind === "notice") {
      return <NoticeRow key={row.key} notice={row.notice} at={row.item.createdAt} />;
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
  // A search result opened the chat: bring its row into view, marked.
  useLayoutEffect(() => {
    if (!focus || shownFocus.current === focus) {
      return;
    }
    const row = listRef.current?.children[rows.findIndex((each) => rowHas(each, focus))];
    if (!row) {
      return;
    }
    shownFocus.current = focus;
    atEnd.current = false;
    row.scrollIntoView?.({ block: "center" });
    row.classList.add("search-focus");
    // As long as the band's animation, also when it does not move.
    setTimeout(() => row.classList.remove("search-focus"), FOCUS_MS);
  }, [focus, rows]);

  if (live && !openRun) {
    list.push(<BotRun key="live" items={[]} bot={bot} live={live} />);
  }

  return (
    <section
      aria-label={t.chat.view.label(bot.name)}
      className="relative flex min-h-0 min-w-0 flex-1 flex-col"
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
        <div className="flex flex-col px-5 pt-5 pb-3">
          {chat.error && (
            <Callout tone="danger" title={t.chat.view.loadFailed}>
              {chat.error}
            </Callout>
          )}
          {chat.items.length > 0 && !chat.complete && (
            <div className="flex justify-center pb-3">
              <Button size="sm" disabled={chat.loading} onClick={loadOlder}>
                {t.chat.view.loadEarlier}
              </Button>
            </div>
          )}
          {chat.items.length === 0 && chat.loading && (
            <p role="status" className="flex items-center gap-2 py-6 text-muted text-sm">
              <LoaderCircle aria-hidden size={14} className="animate-spin" />
              {t.chat.view.loading}
            </p>
          )}
          {chat.items.length === 0 && !chat.loading && !chat.error && !live && (
            <div className="flex flex-col items-center gap-3 py-16 text-center">
              <span className="animate-float">
                <BotAvatar
                  color={bot.color}
                  size={56}
                  mood={moodOf(bot, stopped)}
                  botId={bot.id}
                  starting={bot.state === "launching"}
                />
              </span>
              <p className="font-semibold text-base">{t.chat.view.emptyTitle(bot.name)}</p>
              {bot.role && <p className="max-w-md text-ink-soft text-sm">{bot.role}</p>}
              <p className="max-w-md text-muted text-sm">{t.chat.view.emptyBody}</p>
            </div>
          )}
          <StartReply.Provider value={setReplying}>
            <SeenSince.Provider value={openedAt}>
              <ol
                ref={listRef}
                aria-label={t.chat.view.messages}
                className="offscreen-rows flex flex-col gap-6"
              >
                {list}
              </ol>
            </SeenSince.Provider>
          </StartReply.Provider>
        </div>
      </div>
      <div className="w-full">
        {/* Keyed by bot: each one has its own draft (spec 15.1). */}
        <ChatComposer
          key={bot.id}
          bot={bot}
          files={files}
          stopped={stopped}
          onSent={onSent}
          replying={replying}
          onCancelReply={cancelReply}
        />
      </div>
      {dragging && (
        <div className="pointer-events-none absolute inset-3 grid place-items-center rounded-2xl border-2 border-accent border-dashed bg-canvas/85">
          <p className="flex items-center gap-2 font-medium">
            <Paperclip aria-hidden size={16} />
            {t.chat.view.dropToAttach}
          </p>
        </div>
      )}
    </section>
  );
});
