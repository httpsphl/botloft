// The cheer (spec 31.4): when an agent goes from working to free, the island
// shows "<agent> finished" with the first line of its last reply for a few
// seconds, unless the owner turned the cheer off (31.9).

import { useEffect, useRef, useState } from "react";
import type { BotloftApi } from "../lib/api";
import type { Bot, BotId, BotState } from "../lib/protocol.gen";
import { prefs } from "../shell/prefs";

export interface Cheer {
  bot: Bot;
  /** The first line of what it answered; empty when it said nothing. */
  line: string;
}

export const CHEER_SHOWN_MS = 4000;

export function useCheer(api: BotloftApi, bots: Bot[]): Cheer | null {
  const [cheer, setCheer] = useState<Cheer | null>(null);
  const known = useRef(bots);
  known.current = bots;
  const states = useRef(new Map<BotId, BotState>());

  useEffect(() => {
    for (const bot of bots) {
      if (!states.current.has(bot.id)) {
        states.current.set(bot.id, bot.state);
      }
    }
  }, [bots]);

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | undefined;
    let alive = true;
    const off = api.subscribe((event) => {
      if (event.name !== "bot.state") {
        return;
      }
      const { botId, state } = event.params;
      const before = states.current.get(botId);
      states.current.set(botId, state);
      const bot = known.current.find((have) => have.id === botId);
      if (before !== "busy" || state !== "idle" || !bot || !prefs.islandCheer.get()) {
        return;
      }
      api.call("chat.history", { botId, limit: 5 }).then(
        (newestFirst) => {
          if (!alive) {
            return;
          }
          const reply = newestFirst.find((item) => item.body.kind === "reply");
          const text = reply?.body.kind === "reply" ? reply.body.text : "";
          const line = text.split("\n").find((part) => part.trim() !== "") ?? "";
          clearTimeout(timer);
          setCheer({ bot, line: line.replace(/[*_`#>]/g, "").trim() });
          timer = setTimeout(() => setCheer(null), CHEER_SHOWN_MS);
        },
        () => {},
      );
    });
    return () => {
      alive = false;
      off();
      clearTimeout(timer);
    };
  }, [api]);

  return cheer;
}
