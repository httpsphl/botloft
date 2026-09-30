// The files a bot made, kept current (spec 8.5, 15.1): read when the bot is
// opened, again while it works with the files panel in sight, and when it
// stops, since a scan of its folders is the only way to see what a script
// or a command made. Each read walks the folders, so nobody pays for it
// while the list is out of sight.

import { useCallback, useEffect, useRef, useState } from "react";
import { errorText } from "../../lib/api";
import type { Bot, BotFile } from "../../lib/protocol.gen";
import { useWindowVisible } from "../../shell/visibility";
import { useApi, useApp } from "../../store/context";

/** How often the list is read while the bot works and the panel is in sight. */
const WHILE_WORKING_MS = 4000;

export interface BotFiles {
  /** Newest first. */
  files: BotFile[];
  loading: boolean;
  error: string | null;
  refresh(): Promise<void>;
}

/** `watching`: the files panel is open. */
export function useBotFiles(bot: Pick<Bot, "id" | "state">, watching: boolean): BotFiles {
  const api = useApi();
  const open = useApp((state) => state.connection.kind === "open");
  const [files, setFiles] = useState<BotFile[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  /** Only the newest read counts: a bot opened meanwhile has its own. */
  const latest = useRef(0);
  const botId = bot.id;

  const refresh = useCallback(async () => {
    latest.current += 1;
    const mine = latest.current;
    try {
      const found = await api.call("files.list", { botId });
      if (mine === latest.current) {
        setFiles(found);
        setError(null);
      }
    } catch (failure) {
      if (mine === latest.current) {
        setError(errorText(failure));
      }
    } finally {
      if (mine === latest.current) {
        setLoading(false);
      }
    }
  }, [api, botId]);

  // A new bot, or a new connection: start over.
  useEffect(() => {
    if (!open) {
      return;
    }
    setFiles([]);
    setLoading(true);
    void refresh();
  }, [open, refresh]);

  const working = bot.state === "busy" || bot.state === "needs_approval";
  const wasWorking = useRef(working);
  const inSight = useWindowVisible() && watching;
  useEffect(() => {
    if (!open) {
      return;
    }
    // Once more when it stops: the last file lands just before the end.
    if (wasWorking.current && !working) {
      void refresh();
    }
    wasWorking.current = working;
    if (!working || !inSight) {
      return;
    }
    // Coming into sight mid-turn shows what was made meanwhile.
    void refresh();
    const timer = setInterval(() => void refresh(), WHILE_WORKING_MS);
    return () => clearInterval(timer);
  }, [open, working, inSight, refresh]);

  return { files, loading, error, refresh };
}
