// What the bot did in one turn, under its avatar: replies in markdown,
// tool calls together, approvals, and how the turn ended. The last run
// also shows the reply being written.

import { CircleAlert } from "lucide-react";
import { duration, when } from "../../lib/format";
import type { Bot, ChatItem, TurnItem } from "../../lib/protocol.gen";
import { BotAvatar } from "../bots/BotAvatar";
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
  if (turn.error) {
    return (
      <p className="flex items-center gap-1.5 text-danger text-xs">
        <CircleAlert aria-hidden size={12} />
        The turn stopped: {turn.error.replaceAll("_", " ")}
      </p>
    );
  }
  const cost = turn.costUsd === null ? "" : ` · about $${turn.costUsd.toFixed(2)} of usage`;
  return (
    <p className="text-muted text-xs" title={`Took ${duration(turn.durationMs)}${cost}`}>
      Done in {duration(turn.durationMs)}
    </p>
  );
}

function Working() {
  return (
    <p role="status" className="flex items-center gap-1 py-1 text-muted" aria-label="Working">
      {[0, 150, 300].map((delay) => (
        <span
          key={delay}
          className="h-1.5 w-1.5 animate-pulse rounded-full bg-current"
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
    <li className="flex gap-3 pr-10">
      <BotAvatar color={bot.color} size={28} />
      <div className="flex min-w-0 max-w-[48rem] flex-1 flex-col gap-2">
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
              return <Markdown key={head.id} text={head.body.text} />;
            case "approval":
              return <ApprovalCard key={head.id} approval={head.body} bot={bot} />;
            case "turn":
              return <TurnEnd key={head.id} turn={head.body} />;
            default:
              return null;
          }
        })}
        {live?.draft && (
          <div aria-live="off" className="opacity-90">
            <Markdown text={live.draft} />
          </div>
        )}
        {live && !live.draft && live.working && <Working />}
      </div>
    </li>
  );
}
