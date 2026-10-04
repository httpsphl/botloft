import { Plus } from "lucide-react";
import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { BotStateBadge } from "../bots/BotStateBadge";
import { ChiefBadge, isChief } from "../bots/ChiefBadge";
import { ListAvatar } from "../bots/ListAvatar";

/** The crew's bots as cards; a card opens the bot. */
export function CrewBots({ crew, bots, onNewBot }: { crew: Crew; bots: Bot[]; onNewBot(): void }) {
  const t = useT();
  const selectBot = useApp((state) => state.selectBot);
  if (bots.length === 0) {
    return (
      <div className="rounded-2xl border border-line border-dashed px-6 py-10 text-center">
        <p className="font-medium">{t.crews.bots.empty(crew.name)}</p>
        <p className="mt-1 text-muted text-sm">{t.crews.bots.emptyHint}</p>
        <Button className="mt-4" variant="primary" icon={Plus} onClick={onNewBot}>
          {t.crews.newBot}
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
            className="flex h-full w-full flex-col gap-2 rounded-xl border border-line bg-panel p-3.5 text-left transition-[border-color,transform,box-shadow] duration-200 hover:-translate-y-0.5 hover:border-line-strong hover:shadow-sm"
          >
            <div className="flex w-full items-center gap-2.5">
              <ListAvatar bot={bot} crewPaused={crew.paused} size={32} />
              <div className="min-w-0 flex-1">
                <p className="flex items-center gap-1.5">
                  <span className="truncate font-semibold">{bot.name}</span>
                  {isChief(bot, crew) && <ChiefBadge crew={crew} compact />}
                </p>
                <p className="truncate font-mono text-muted text-xs">@{bot.handle}</p>
              </div>
            </div>
            <p className="line-clamp-2 min-h-10 text-ink-soft text-sm">
              {bot.role || t.crews.bots.noRole}
            </p>
            <BotStateBadge bot={bot} crewPaused={crew.paused} />
          </button>
        </li>
      ))}
    </ul>
  );
}
