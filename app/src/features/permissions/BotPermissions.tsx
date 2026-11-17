// What one agent may do without asking, in its details (spec 15.1):
// "Always allow", its browser's sites and other crews, each item with its X,
// and the way to the Permissions page on it. Its desktop and the owner's
// Chrome are rows of their own below (BotShortcuts).

import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { AllowRules } from "../bots/AllowRules";
import { CrewAccessList } from "../bots/CrewAccessList";
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
    <section aria-label={words.section} className="flex flex-col gap-2">
      <div className="flex items-center justify-between gap-2">
        <h3 className="font-semibold text-sm">{words.section}</h3>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => openPermissions({ tab: "rules", botId: bot.id })}
        >
          {words.seeAll}
        </Button>
      </div>
      <dl className="m-0 flex flex-col gap-4">
        <AllowRules bot={bot} />
        <BotSites bot={bot} />
        <CrewAccessList bot={bot} />
      </dl>
    </section>
  );
}
