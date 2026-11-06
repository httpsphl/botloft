// The round buttons at the end of the composer: send, and, while the bot
// works, stop what it does (spec 9.2, 15.1). With nothing written, stop
// takes the place of send. A message still waiting in line runs after the
// stop, so "stop, then do this instead" is a stop and a message.

import { ArrowUp, Square } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { attempt } from "../../ui/toast";

/** Whether the owner can stop `bot` now: it works, and its agent can be stopped. */
export function canStop(bot: Bot): boolean {
  // Only Claude Code answers a request to stop a turn (spec 30).
  return bot.agent === "claude" && (bot.state === "busy" || bot.state === "needs_approval");
}

function StopButton({ bot }: { bot: Bot }) {
  const api = useApi();
  const putBot = useApp((state) => state.putBot);
  const t = useT().chat.composer;
  const [stopping, setStopping] = useState(false);

  const stop = async () => {
    setStopping(true);
    await attempt(t.stopFailed, async () => {
      putBot(await api.call("bots.interrupt", { botId: bot.id }));
    });
    setStopping(false);
  };

  return (
    <button
      type="button"
      aria-label={t.stop(bot.name)}
      title={t.stopHint(bot.name)}
      disabled={stopping}
      onClick={() => void stop()}
      className="grid h-8 w-8 shrink-0 animate-fade place-items-center rounded-full bg-ink text-canvas transition-[opacity,transform] duration-150 hover:opacity-85 active:scale-90 disabled:opacity-50"
    >
      <Square aria-hidden size={11} fill="currentColor" strokeWidth={0} />
    </button>
  );
}

export function SendButtons({
  bot,
  empty,
  disabled,
}: {
  bot: Bot;
  /** Nothing written and no file picked. */
  empty: boolean;
  disabled: boolean;
}) {
  const t = useT();
  const stoppable = canStop(bot);
  return (
    <>
      {stoppable && <StopButton bot={bot} />}
      {(!stoppable || !empty) && (
        <button
          type="submit"
          aria-label={t.chat.composer.send}
          disabled={disabled}
          className="group/send grid h-8 w-8 shrink-0 place-items-center rounded-full bg-accent text-canvas transition-[opacity,transform,background-color] duration-150 hover:opacity-90 active:scale-90 disabled:bg-line-strong disabled:opacity-60"
        >
          <ArrowUp
            aria-hidden
            size={16}
            strokeWidth={2.25}
            className="transition-transform duration-200 group-hover/send:-translate-y-0.5"
          />
        </button>
      )}
    </>
  );
}
