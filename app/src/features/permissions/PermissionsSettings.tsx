// Everything the owner let each agent do, in one place (spec 15.1): a part
// of Settings with four tabs (the desktop, the browsers' sites, "Always
// allow" and other crews), every agent or one, each item with its X.

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { BotId } from "../../lib/protocol.gen";
import type { PermissionsTab } from "../../store/app";
import { useApp } from "../../store/context";
import { Select } from "../../ui/Select";
import { Tabs, tabId } from "../../ui/Tabs";
import { AgentGroups, counted, groupByAgent } from "./AgentGroups";
import { DesktopTab } from "./DesktopTab";
import { CrewAccessRow, RuleRow, SiteRow } from "./rows";
import { usePermissions } from "./usePermissions";

const EVERYONE = "";

export function PermissionsSettings() {
  const t = useT();
  const words = t.permissions;
  const all = usePermissions();
  const bots = useApp((state) => state.bots);
  const crews = useApp((state) => state.crews);
  const at = useApp((state) => state.permissionsAt);
  const clearAt = useApp((state) => state.clearPermissionsAt);
  const [tab, setTab] = useState<PermissionsTab>(at?.tab ?? "desktop");
  const [only, setOnly] = useState<BotId | null>(at?.botId ?? null);
  // Opened from an agent's details or its desktop panel: there, then done.
  useEffect(() => {
    if (at) {
      setTab(at.tab);
      setOnly(at.botId);
      clearAt();
    }
  }, [at, clearAt]);

  const crewName = (botId: BotId) => crews[bots[botId]?.crewId ?? ""]?.name ?? "";
  const agents = Object.values(bots)
    .filter((bot) => bot.archivedAt === null)
    .sort((a, b) => crewName(a.id).localeCompare(crewName(b.id)) || a.name.localeCompare(b.name));
  const desktop = groupByAgent(all?.desktop ?? [], bots, crews, only);
  const sites = groupByAgent(all?.sites ?? [], bots, crews, only);
  const rules = groupByAgent(all?.rules ?? [], bots, crews, only);
  const access = groupByAgent(all?.crewAccess ?? [], bots, crews, only);
  const label = (name: string, count: number) => (count > 0 ? `${name} (${count})` : name);

  return (
    <div className="flex flex-col gap-4">
      <p className="text-ink-soft text-sm leading-relaxed">{words.intro}</p>
      <div className="w-full max-w-xs">
        <Select<BotId>
          label={words.agent}
          value={only ?? EVERYONE}
          options={[
            { value: EVERYONE, label: words.everyone },
            ...agents.map((bot) => ({
              value: bot.id,
              label: [bot.name, crewName(bot.id)].filter(Boolean).join(" · "),
            })),
          ]}
          onChange={(value) => setOnly(value === EVERYONE ? null : value)}
        />
      </div>
      <Tabs<PermissionsTab>
        label={t.account.settings.permissions}
        tabs={[
          { id: "desktop", label: label(words.tabs.desktop, counted(desktop)) },
          { id: "sites", label: label(words.tabs.sites, counted(sites)) },
          { id: "rules", label: label(words.tabs.rules, counted(rules)) },
          { id: "crews", label: label(words.tabs.crews, counted(access)) },
        ]}
        value={tab}
        onChange={setTab}
      />
      <div role="tabpanel" aria-labelledby={tabId(tab)} className="flex flex-col gap-3">
        {all !== null && tab === "desktop" && (
          <DesktopTab groups={desktop} agents={agents} only={only} />
        )}
        {all !== null && tab === "sites" && (
          <>
            <AgentGroups
              groups={sites}
              empty={words.empty.sites}
              keyOf={(site) => site.site}
              row={(_, site) => <SiteRow site={site} />}
            />
            <p className="text-muted text-xs leading-relaxed">{words.sitesModes}</p>
          </>
        )}
        {all !== null && tab === "rules" && (
          <AgentGroups
            groups={rules}
            empty={words.empty.rules}
            keyOf={(rule) => rule.id}
            row={(_, rule) => <RuleRow rule={rule} />}
          />
        )}
        {all !== null && tab === "crews" && (
          <AgentGroups
            groups={access}
            empty={words.empty.crews}
            keyOf={(each) => each.id}
            row={(_, each) => <CrewAccessRow access={each} />}
          />
        )}
      </div>
    </div>
  );
}
