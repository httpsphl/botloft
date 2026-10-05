// The owner's connected tools (spec 25): the list, who uses each and how
// they stand, read when shown and kept current by `mcp.servers` and
// `bot.mcp`.

import { useEffect, useState } from "react";
import type { BotMcp, McpOverview } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

const NONE: McpOverview = { servers: [], bots: [] };

/** `null` until the daemon has answered; an older daemon has no tools. */
export function useConnections(): McpOverview | null {
  const api = useApi();
  const [overview, setOverview] = useState<McpOverview | null>(null);
  useEffect(() => {
    let alive = true;
    const stop = api.subscribe((event) => {
      if (event.name === "mcp.servers") {
        setOverview(event.params);
      } else if (event.name === "bot.mcp") {
        const bot = event.params;
        setOverview((current) => (current ? withBot(current, bot) : current));
      }
    });
    api.call("mcp.servers").then(
      (listed) => alive && setOverview((current) => current ?? listed),
      () => alive && setOverview((current) => current ?? NONE),
    );
    return () => {
      alive = false;
      stop();
    };
  }, [api]);
  return overview;
}

/** `bot` replaces what the overview says about that bot. */
function withBot(overview: McpOverview, bot: BotMcp): McpOverview {
  const others = overview.bots.filter((each) => each.botId !== bot.botId);
  return { ...overview, bots: bot.serverIds.length > 0 ? [...others, bot] : others };
}
