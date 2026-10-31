// The island's quick chat with one agent (spec 31.4): its last items, read
// once from its chat and kept by `chat.item`, and the text it is writing.

import { useEffect, useState } from "react";
import type { BotloftApi } from "../lib/api";
import type { BotId, ChatItem } from "../lib/protocol.gen";

/** How many items the island reads back. */
const HISTORY = 20;

export function useTalk(
  api: BotloftApi,
  botId: BotId | null,
): { items: ChatItem[]; writing: string } {
  const [items, setItems] = useState<ChatItem[]>([]);
  const [writing, setWriting] = useState("");

  useEffect(() => {
    setItems([]);
    setWriting("");
    if (!botId) {
      return;
    }
    let alive = true;
    api.call("chat.history", { botId, limit: HISTORY }).then(
      (newestFirst) => alive && setItems([...newestFirst].reverse()),
      () => {},
    );
    const off = api.subscribe((event) => {
      if (event.name === "chat.delta" && event.params.botId === botId) {
        setWriting((now) => now + event.params.text);
      } else if (event.name === "chat.item" && event.params.item.botId === botId) {
        const item = event.params.item;
        setItems((now) => {
          const at = now.findIndex((have) => have.id === item.id);
          return at >= 0 ? now.map((have, i) => (i === at ? item : have)) : [...now, item];
        });
        if (item.body.kind === "reply" || item.body.kind === "turn") {
          setWriting("");
        }
      }
    });
    return () => {
      alive = false;
      off();
    };
  }, [api, botId]);

  return { items, writing };
}
