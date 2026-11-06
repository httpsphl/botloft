// What the bot may see and use on the owner's desktop (spec 24.10): its
// grants, read when needed and kept current by `bot.desktop`.

import { useEffect, useState } from "react";
import type { Bot, DesktopGrant } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

/** The bot's grants, read when shown and kept current by `bot.desktop`. */
export function useDesktopGrants(botId: Bot["id"]): DesktopGrant[] | null {
  const api = useApi();
  const [grants, setGrants] = useState<DesktopGrant[] | null>(null);
  useEffect(() => {
    let alive = true;
    setGrants(null);
    const stop = api.subscribe((event) => {
      if (event.name === "bot.desktop" && event.params.botId === botId) {
        setGrants(event.params.grants);
      }
    });
    api.call("desktop.grants", { botId }).then(
      (listed) => alive && setGrants((current) => current ?? listed),
      // An older daemon has no desktop: the list stays empty.
      () => alive && setGrants([]),
    );
    return () => {
      alive = false;
      stop();
    };
  }, [api, botId]);
  return grants;
}
