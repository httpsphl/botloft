// What the owner let one agent do, together in its details (spec 15.1):
// "Always allow", its browser's sites, other crews and the desktop, each
// item with its X, and the way to every agent's on the Permissions page.

import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { AllowRules } from "../bots/AllowRules";
import { CrewAccessList } from "../bots/CrewAccessList";
import { DesktopGrants } from "../desktop/DesktopGrants";
import { SiteRow } from "./rows";
import { usePermissions } from "./usePermissions";

function BotSites({ bot }: { bot: Bot }) {
  const words = useT().permissions;
  const all = usePermissions();
  const sites = all?.sites.filter((site) => site.botId === bot.id) ?? null;
  return (
    <div>
      <dt className="text-muted text-xs">{words.sites}</dt>
      <dd className="mt-1">
        {sites !== null && sites.length === 0 && (
          <p className="text-ink-soft text-xs leading-relaxed">{words.sitesNone(bot.name)}</p>
        )}
        {sites !== null && sites.length > 0 && (
          <ul className="flex flex-col gap-1.5">
            {sites.map((site) => (
              <SiteRow key={site.site} site={site} />
            ))}
          </ul>
        )}
      </dd>
    </div>
  );
}

export function BotPermissions({ bot }: { bot: Bot }) {
  const words = useT().permissions;
  const openPermissions = useApp((state) => state.openPermissions);
  return (
    <div className="rounded-xl border border-line p-3">
      <dt className="flex items-center justify-between gap-2">
        <span className="font-semibold text-muted text-xs uppercase tracking-[0.12em]">
          {words.section}
        </span>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => openPermissions({ tab: "desktop", botId: bot.id })}
        >
          {words.seeAll}
        </Button>
      </dt>
      <dd className="mt-2">
        <dl className="flex flex-col gap-4">
          <AllowRules bot={bot} />
          <BotSites bot={bot} />
          <CrewAccessList bot={bot} />
          <DesktopGrants bot={bot} />
        </dl>
      </dd>
    </div>
  );
}
