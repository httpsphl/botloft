// What the bot did in one turn, on the left of its chat: replies in
// balloons, tool calls together, approvals, and how the turn ended. The
// header already says whose chat it is, so a run has no name over it. The
// last run also shows the reply being written.

import { CircleAlert } from "lucide-react";
import { memo, type ReactNode, useMemo } from "react";
import { useT } from "../../i18n";
import { duration, tokens, usedTokens } from "../../lib/format";
import type { Bot, ChatItem, TurnItem } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { useArrival } from "../../ui/motion";
import { BotAvatar, moodOf } from "../bots/BotAvatar";
import { QuestionCard } from "../questions/QuestionCard";
import { ApprovalCard } from "./ApprovalCard";
import { KeyRequestCard } from "./KeyRequestCard";
import { Markdown } from "./Markdown";
import { MemoryNote } from "./MemoryNote";
import { memoryOf, memoryPlaces } from "./memory";
import { Repliable } from "./Replying";
import { groupRepeat, repeatCounts } from "./repeats";
import { runParts, splitDraft } from "./rows";
import { SharedFiles } from "./SharedFiles";
import { isSharedFiles } from "./shared";
import { ToolGroup } from "./ToolGroup";
import { TurnMedia } from "./TurnMedia";

export interface Live {
  /** Text of the reply being written. */
  draft: string;
  /** The bot is in a turn. */
  working: boolean;
}

function TurnEnd({ turn }: { turn: TurnItem }) {
  const t = useT();
  if (turn.error) {
    return (
      <p className="flex items-center gap-1.5 px-1 text-danger text-xs">
        <CircleAlert aria-hidden size={12} />
        {t.chat.run.stopped(turn.error.replaceAll("_", " "))}
      </p>
    );
  }
  const time = duration(turn.durationMs);
  const used = turn.tokens;
  const detail = used && {
    read: tokens(used.input + used.cacheWrite),
    wrote: tokens(used.output),
  };
  const spent = used && tokens(usedTokens(used));
  return (
    <p className="px-1 text-muted text-xs" title={t.chat.run.took(time, detail)}>
      {turn.quiet
        ? t.chat.run.stoppedByBotloft(time, spent)
        : turn.stopped
          ? t.chat.run.stoppedByOwner(time, spent)
          : t.chat.run.done(time, spent)}
    </p>
  );
}

/**
 * A card of the run, marked with its item for the chat to find it
 * (`WaitingPill`); it animates in if it arrived while the owner looked.
 */
function Arriving({ id, at, children }: { id: string; at: number; children: ReactNode }) {
  const arrival = useArrival(at);
  return (
    <div data-item={id} className={arrival || undefined}>
      {children}
    </div>
  );
}

/** A balloon of the bot's: what it wrote, or the dots while it thinks. */
const BALLOON = "chat-bubble w-fit max-w-full rounded-bubble bg-bubble px-4 py-2.5";

/** The bot thinking: its mascot, moving with it, and three dots in a balloon. */
function Working({ bot }: { bot: Bot }) {
  const t = useT();
  return (
    <div
      role="status"
      aria-label={t.chat.run.working}
      className="flex w-fit animate-fade items-center gap-2 rounded-bubble bg-bubble py-1.5 pr-4 pl-2 text-muted"
    >
      <BotAvatar color={bot.color} size={24} mood={moodOf(bot)} />
      <span className="flex items-center gap-1">
        {[0, 160, 320].map((delay) => (
          <span
            key={delay}
            className="h-1.5 w-1.5 animate-dot rounded-full bg-current"
            style={{ animationDelay: `${delay}ms` }}
          />
        ))}
      </span>
    </div>
  );
}

function Draft({ text }: { text: string }) {
  const [done, writing] = splitDraft(text);
  return (
    // chat-md.css spaces the two parts as one reply; `data-reply` is where
    // following a long one stops (follow.ts).
    <div aria-live="off" data-reply className={`chat-draft animate-fade ${BALLOON}`}>
      {done && <Markdown text={done} />}
      <Markdown text={writing} streaming />
    </div>
  );
}

const sameItems = (a: ChatItem[], b: ChatItem[]) =>
  a.length === b.length && a.every((item, index) => item === b[index]);

/**
 * Re-renders only when something it shows changed: finished runs stay put
 * while the last one streams.
 */
export const BotRun = memo(
  BotRunView,
  (before, after) =>
    before.bot === after.bot &&
    before.live?.draft === after.live?.draft &&
    before.live?.working === after.live?.working &&
    sameItems(before.items, after.items),
);

function BotRunView({
  items,
  bot,
  live,
}: {
  items: ChatItem[];
  bot: Bot;
  live?: Live | undefined;
}) {
  const first = items[0];
  const crew = useApp((state) => state.crews[bot.crewId]);
  const places = useMemo(() => memoryPlaces(bot, crew), [bot, crew]);
  const memory = (item: ChatItem) => memoryOf(item, places, bot.workspace);
  const parts = runParts(items, (item) => memory(item) !== null);
  // The same call counts across the whole turn, not one group (repeats.ts).
  const repeats = useMemo(() => repeatCounts(items), [items]);
  // The bot is still on the last group of calls until it writes or stops.
  const busy = Boolean(live?.working && !live.draft);
  const tail = parts.at(-1)?.[0];
  // Its line already says it is working.
  const onCalls = tail?.body.kind === "tool" && !isSharedFiles(tail) && !memory(tail);
  return (
    // A turn shows up as the bot starts it, still empty; what it does then
    // arrives inside, so a run with items never animates as a whole. The
    // room on the right is for a balloon's time and buttons (Replying.tsx).
    <li aria-label={bot.name} className={`flex pr-14 lg:pr-32 ${first ? "" : "animate-rise"}`}>
      <div className="flex min-w-0 flex-1 flex-col gap-1.5">
        {parts.map((part, index) => {
          const head = part[0] as ChatItem;
          switch (head.body.kind) {
            case "tool": {
              const owner = memory(head);
              if (owner) {
                return (
                  <MemoryNote
                    key={head.id}
                    tool={head.body}
                    createdAt={head.createdAt}
                    owner={owner}
                    name={owner === "bot" ? bot.name : (crew?.name ?? "")}
                    color={owner === "bot" ? bot.color : undefined}
                    bot={owner === "bot" ? bot : undefined}
                  />
                );
              }
              return isSharedFiles(head) ? (
                <SharedFiles key={head.id} item={head} botId={bot.id} bot={bot.name} />
              ) : (
                <ToolGroup
                  key={head.id}
                  items={part}
                  active={busy && index === parts.length - 1}
                  repeat={groupRepeat(part, repeats)}
                  bot={bot.name}
                />
              );
            }
            case "reply":
              // It was already on screen as it was written.
              return (
                <div key={head.id} data-reply>
                  <Repliable
                    target={{ itemId: head.id, who: bot.name, text: head.body.text }}
                    at={head.createdAt}
                    react
                  >
                    <div className={BALLOON}>
                      <Markdown text={head.body.text} />
                    </div>
                  </Repliable>
                </div>
              );
            case "approval":
              return (
                <Arriving key={head.id} id={head.id} at={head.createdAt}>
                  <ApprovalCard approval={head.body} bot={bot} />
                </Arriving>
              );
            case "question":
              return (
                <Arriving key={head.id} id={head.id} at={head.createdAt}>
                  <QuestionCard question={head.body.question} bot={bot.name} />
                </Arriving>
              );
            case "key_request":
              return (
                <Arriving key={head.id} id={head.id} at={head.createdAt}>
                  <KeyRequestCard id={head.id} request={head.body} bot={bot} />
                </Arriving>
              );
            case "turn":
              return (
                <Arriving key={head.id} id={head.id} at={head.createdAt}>
                  <div className="flex flex-col gap-1.5">
                    <TurnMedia items={items} turn={head} botId={bot.id} bot={bot.name} />
                    <TurnEnd turn={head.body} />
                  </div>
                </Arriving>
              );
            default:
              return null;
          }
        })}
        {live?.draft && <Draft text={live.draft} />}
        {busy && !onCalls && <Working bot={bot} />}
      </div>
    </li>
  );
}
