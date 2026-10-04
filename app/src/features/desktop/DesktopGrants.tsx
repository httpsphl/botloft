// What the bot may see and use on the owner's desktop (spec 24.10), in its
// details: each app the owner let it see, with a button that takes it back.

import { Monitor, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, DesktopGrant } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { RealInput } from "./RealInput";

/** The bot's grants, read when shown and kept current by `bot.desktop`. */
function useDesktopGrants(botId: Bot["id"]): DesktopGrant[] | null {
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
  const api = useApi();
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
            {grants.map((grant) => {
              const app = grant.scope === "desktop" ? words.whole : (grant.appName ?? "");
              return (
                <li
                  key={grant.id}
                  className="flex items-start gap-2 rounded-lg border border-line bg-sunken py-1.5 pr-1 pl-2.5"
                >
                  <Monitor aria-hidden size={14} className="mt-0.5 shrink-0 text-muted" />
                  <span className="min-w-0 flex-1">
                    <span className="block text-xs">{app}</span>
                    <span className="block text-ink-soft text-xs">
                      {grant.level === "act" ? words.act : words.see}
                    </span>
                    {grant.level === "act" && <RealInput bot={bot} grant={grant} app={app} />}
                    {grant.appPath && (
                      <span
                        className="block break-all font-mono text-muted text-xs"
                        data-selectable
                      >
                        {grant.appPath}
                      </span>
                    )}
                  </span>
                  <Button
                    variant="ghost"
                    size="sm"
                    icon={X}
                    label={words.remove(app)}
                    onClick={() =>
                      attempt(words.removeFailed, () =>
                        api.call("desktop.revoke", { grantId: grant.id }),
                      )
                    }
                  />
                </li>
              );
            })}
          </ul>
        )}
      </dd>
    </div>
  );
}
