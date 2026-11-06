// Settings, "Agents' keys" (spec 32.5): one agent at a time, chosen at the
// top, with its keys by name, replace, delete and add. An agent's details
// open it on that agent.

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { BotId } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Select } from "../../ui/Select";
import { BotAvatar } from "../bots/BotAvatar";
import { AgentKeys } from "./AgentKeys";

export function KeysSettings() {
  const t = useT();
  const words = t.keys.page;
  const bots = useApp((state) => state.bots);
  const crews = useApp((state) => state.crews);
  const at = useApp((state) => state.settingsBotId);
  const clearAt = useApp((state) => state.clearSettingsBotId);
  const crewName = (botId: BotId) => crews[bots[botId]?.crewId ?? ""]?.name ?? "";
  const agents = Object.values(bots)
    .filter((bot) => bot.archivedAt === null)
    .sort((a, b) => crewName(a.id).localeCompare(crewName(b.id)) || a.name.localeCompare(b.name));
  const [chosen, setChosen] = useState<BotId | null>(at);
  // Opened from an agent's details: on it, then done.
  useEffect(() => {
    if (at) {
      setChosen(at);
      clearAt();
    }
  }, [at, clearAt]);
  const bot = (chosen && bots[chosen]) || agents[0];

  if (!bot) {
    return <p className="text-ink-soft text-sm leading-relaxed">{words.noAgents}</p>;
  }
  return (
    <div className="flex flex-col gap-4">
      <p className="text-ink-soft text-sm leading-relaxed">{words.intro}</p>
      <div className="w-full max-w-xs">
        <Select<BotId>
          label={t.permissions.agent}
          value={bot.id}
          options={agents.map((each) => ({
            value: each.id,
            label: [each.name, crewName(each.id)].filter(Boolean).join(" · "),
          }))}
          onChange={setChosen}
        />
      </div>
      <section aria-label={bot.name} className="flex flex-col gap-3">
        <h3 className="flex min-w-0 items-center gap-2 font-medium text-sm">
          <BotAvatar color={bot.color} size={20} still botId={bot.id} />
          <span className="truncate">{bot.name}</span>
        </h3>
        <dl className="m-0">
          <AgentKeys key={bot.id} bot={bot} titled={false} />
        </dl>
      </section>
    </div>
  );
}
