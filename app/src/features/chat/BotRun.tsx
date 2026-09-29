// What the bot did in one turn, under its avatar: replies in markdown,
// tool calls together, approvals, and how the turn ended. The last run
// also shows the reply being written.

import { CircleAlert } from "lucide-react";
import type { ReactNode } from "react";
import { useT } from "../../i18n";
import { duration, when } from "../../lib/format";
import type { Bot, ChatItem, TurnItem } from "../../lib/protocol.gen";
import { useArrival } from "../../ui/motion";
import { BotAvatar, moodOf } from "../bots/BotAvatar";
import { ApprovalCard } from "./ApprovalCard";
import { Markdown } from "./Markdown";
import { runParts } from "./rows";
import { ToolLines } from "./ToolLines";

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
      <p className="flex items-center gap-1.5 text-danger text-xs">
        <CircleAlert aria-hidden size={12} />
        {t.chat.run.stopped(turn.error.replaceAll("_", " "))}
      </p>
    );
  }
  const time = duration(turn.durationMs);
  return (
    <p className="text-muted text-xs" title={t.chat.run.took(time, turn.costUsd)}>
      {t.chat.run.done(time)}
    </p>
  );
}

/** Animates its content in if it arrived while the owner looked. */
function Arriving({ at, children }: { at: number; children: ReactNode }) {
  const arrival = useArrival(at);
  return arrival ? <div className={arrival}>{children}</div> : children;
}

function Working() {
  const t = useT();
  return (
    <p
      role="status"
      className="flex items-center gap-1 py-1 text-muted"
      aria-label={t.chat.run.working}
    >
      {[0, 160, 320].map((delay) => (
        <span
          key={delay}
          className="h-1.5 w-1.5 animate-dot rounded-full bg-current"
          style={{ animationDelay: `${delay}ms` }}
        />
      ))}
    </p>
  );
}

export function BotRun({
  items,
  bot,
  live,
}: {
  items: ChatItem[];
  bot: Bot;
  live?: Live | undefined;
}) {
  const first = items[0];
  return (
    // A turn shows up as the bot starts it, still empty; what it does then
    // arrives inside, so a run with items never animates as a whole.
    <li className={`flex gap-3 ${first ? "" : "animate-rise"}`}>
      {/* Only the run in progress moves, like the bot it stands for. */}
      <BotAvatar color={bot.color} size={28} mood={live ? moodOf(bot) : undefined} />
      <div className="flex min-w-0 flex-1 flex-col gap-2">
        <div className="flex items-center gap-2 text-sm">
          <span className="font-semibold">{bot.name}</span>
          {first && (
            <time className="text-muted text-xs" dateTime={new Date(first.createdAt).toISOString()}>
              {when(first.createdAt)}
            </time>
          )}
        </div>
        {runParts(items).map((part) => {
          const head = part[0] as ChatItem;
          switch (head.body.kind) {
            case "tool":
              return <ToolLines key={head.id} items={part} />;
            case "reply":
              // It was already on screen as it was written.
              return <Markdown key={head.id} text={head.body.text} />;
            case "approval":
              return (
                <Arriving key={head.id} at={head.createdAt}>
                  <ApprovalCard approval={head.body} bot={bot} />
                </Arriving>
              );
            case "turn":
              return (
                <Arriving key={head.id} at={head.createdAt}>
                  <TurnEnd turn={head.body} />
                </Arriving>
              );
            default:
              return null;
          }
        })}
        {live?.draft && (
          <div aria-live="off" className="animate-fade opacity-90">
            <Markdown text={live.draft} streaming />
          </div>
        )}
        {live && !live.draft && live.working && <Working />}
      </div>
    </li>
  );
}
