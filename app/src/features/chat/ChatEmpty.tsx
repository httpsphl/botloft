// A chat with nothing in it yet: the bot, its role, and how to start. The
// Maker invites the owner to tell it about their work (spec 36.3).

import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { makerBot } from "../../store/app";
import { useApp } from "../../store/context";
import { BotAvatar, moodOf } from "../bots/BotAvatar";

export function ChatEmpty({ bot, stopped }: { bot: Bot; stopped: boolean }) {
  const t = useT();
  const isMaker = useApp((state) => makerBot(state)?.id === bot.id);
  return (
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
      {isMaker ? (
        <>
          <p className="max-w-md font-semibold text-base">{t.maker.emptyTitle}</p>
          <p className="max-w-md text-ink-soft text-sm leading-relaxed">{t.maker.emptyBody}</p>
        </>
      ) : (
        <>
          <p className="font-semibold text-base">{t.chat.view.emptyTitle(bot.name)}</p>
          {bot.role && <p className="max-w-md text-ink-soft text-sm">{bot.role}</p>}
          <p className="max-w-md text-muted text-sm">{t.chat.view.emptyBody}</p>
        </>
      )}
    </div>
  );
}
