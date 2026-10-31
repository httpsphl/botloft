// What each working agent is doing right now (spec 31.4): the items of its
// turn so far and the text it is writing. Read once per agent from its
// chat, then kept by the daemon's `chat.item` and `chat.delta`.

import { useEffect, useState } from "react";
import type { BotloftApi } from "../lib/api";
import type { BotId, ChatItem } from "../lib/protocol.gen";

export interface Live {
  /** The items since the turn began, oldest first. */
  items: ChatItem[];
  /** Text written after the last item, not yet a reply. */
  writing: string;
}

/** How far back the island reads to find where the turn began. */
const HISTORY = 30;

/** The items after the last finished turn or the owner's message. */
function thisTurn(items: ChatItem[]): ChatItem[] {
  let start = 0;
  items.forEach((item, at) => {
    if (item.body.kind === "turn" || item.body.kind === "inbound") {
      start = at + 1;
    }
  });
  return items.slice(start);
}

export function useLive(api: BotloftApi, botIds: BotId[]): Map<BotId, Live> {
  const [live, setLive] = useState<Map<BotId, Live>>(new Map());
  const key = botIds.join(",");

  useEffect(() => {
    const ids = key === "" ? [] : (key.split(",") as BotId[]);
    let alive = true;
    const set = (botId: BotId, change: (now: Live) => Live) =>
      setLive((before) => {
        const now = before.get(botId);
        if (!now) {
          return before;
        }
        return new Map(before).set(botId, change(now));
      });
    setLive((before) => {
      const kept = new Map<BotId, Live>();
      for (const id of ids) {
        kept.set(id, before.get(id) ?? { items: [], writing: "" });
      }
      return kept;
    });
    for (const botId of ids) {
      api.call("chat.history", { botId, limit: HISTORY }).then(
        (newestFirst) => {
          if (alive) {
            set(botId, (now) => ({ ...now, items: thisTurn([...newestFirst].reverse()) }));
          }
        },
        () => {},
      );
    }
    const off = api.subscribe((event) => {
      if (event.name === "chat.delta") {
        set(event.params.botId, (now) => ({ ...now, writing: now.writing + event.params.text }));
      } else if (event.name === "chat.item") {
        const item = event.params.item;
        set(item.botId, (now) => {
          const at = now.items.findIndex((have) => have.id === item.id);
          const items =
            at >= 0 ? now.items.map((have, i) => (i === at ? item : have)) : [...now.items, item];
          return { items: thisTurn(items), writing: at >= 0 ? now.writing : "" };
        });
      }
    });
    return () => {
      alive = false;
      off();
    };
  }, [api, key]);

  return live;
}
