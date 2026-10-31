// What the bot may see and use on the owner's desktop (spec 24.10), in its
// details: each app the owner let it see, with a button that takes it back.

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, DesktopGrant } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { DesktopGrantRow } from "../permissions/rows";
import { WholeDesktop } from "./WholeDesktop";

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

export function DesktopGrants({ bot }: { bot: Bot }) {
  const words = useT().desktop.grants;
  const grants = useDesktopGrants(bot.id);
  return (
    <div>
      <dt className="text-muted text-xs">{words.title}</dt>
      <dd className="mt-1">
        {grants !== null && grants.length === 0 && (
          <p className="text-ink-soft text-xs leading-relaxed">{words.none(bot.name)}</p>
        )}
        {grants !== null && grants.length > 0 && (
          <ul className="flex flex-col gap-1.5">
            {grants.map((grant) => (
              <DesktopGrantRow key={grant.id} bot={bot} grant={grant} />
            ))}
          </ul>
        )}
        {grants !== null && !grants.some((grant) => grant.scope === "desktop") && (
          <div className="mt-2">
            <WholeDesktop bot={bot} />
          </div>
        )}
      </dd>
    </div>
  );
}
