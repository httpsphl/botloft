import { Plus } from "lucide-react";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { BotAvatar } from "../bots/BotAvatar";
import { BotStateBadge } from "../bots/BotStateBadge";

/** The crew's bots as cards; a card opens the bot. */
export function CrewBots({ crew, bots, onNewBot }: { crew: Crew; bots: Bot[]; onNewBot(): void }) {
  const selectBot = useApp((state) => state.selectBot);
  if (bots.length === 0) {
    return (
      <div className="border border-line border-dashed px-6 py-10 text-center">
        <p className="font-medium">No bots in {crew.name} yet.</p>
        <p className="mt-1 text-muted text-sm">
          A bot is a Claude Code session that keeps running, with its own folder and role.
        </p>
        <Button className="mt-4" variant="primary" icon={Plus} onClick={onNewBot}>
          New bot
        </Button>
      </div>
    );
  }
  return (
    <ul className="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-3">
      {bots.map((bot) => (
        <li key={bot.id}>
          <button
            type="button"
            onClick={() => selectBot(bot.id)}
            className="flex h-full w-full flex-col gap-2 border border-line bg-panel p-3 text-left hover:border-line-strong"
          >
            <div className="flex w-full items-center gap-2.5">
              <BotAvatar color={bot.color} size={32} />
              <div className="min-w-0 flex-1">
                <p className="truncate font-semibold">{bot.name}</p>
                <p className="truncate font-mono text-muted text-xs">@{bot.handle}</p>
              </div>
            </div>
            <p className="line-clamp-2 min-h-10 text-ink-soft text-sm">
              {bot.role || "No role yet."}
            </p>
            <BotStateBadge bot={bot} crewPaused={crew.paused} />
          </button>
        </li>
      ))}
    </ul>
  );
}
