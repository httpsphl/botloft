// Which agent acts in the owner's Chrome now (spec 37.4): read once, then
// followed by `chrome.turn`.

import { useEffect, useState } from "react";
import type { BotId } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

export function useChromeTurn(): BotId | null {
  const api = useApi();
  const [holder, setHolder] = useState<BotId | null>(null);
  useEffect(() => {
    let alive = true;
    api.call("chrome.status").then(
      (turn) => alive && setHolder(turn.botId),
      // An older daemon has no Chrome: nobody uses it.
      () => {},
    );
    const stop = api.subscribe((event) => {
      if (event.name === "chrome.turn") {
        setHolder(event.params.botId);
      }
    });
    return () => {
      alive = false;
      stop();
    };
  }, [api]);
  return holder;
}
