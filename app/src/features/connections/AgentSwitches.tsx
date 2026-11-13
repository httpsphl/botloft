// A switch per agent, crew by crew: who may use a tool (spec 34.7) or a
// site login (spec 35.4).

import { useShallow } from "zustand/react/shallow";
import type { Bot } from "../../lib/protocol.gen";
import { crewList } from "../../store/app";
import { useApp } from "../../store/context";
import { Switch } from "../../ui/Switch";
import { BotAvatar } from "../bots/BotAvatar";

export function AgentSwitches({
  bots,
  uses,
  onChange,
  label,
  empty,
  locked,
}: {
  /** The agents that may have it, archived ones left out by the caller. */
  bots: Bot[];
  uses(bot: Bot): boolean;
  onChange(bot: Bot, on: boolean): unknown;
  /** The switch's name for screen readers: "Scout can use it". */
  label(bot: Bot): string;
  empty: string;
  /** Shown but not changed one by one: given to every agent. */
  locked?: boolean;
}) {
  const crews = useApp(useShallow(crewList));
  if (bots.length === 0) {
    return <p className="text-muted text-sm">{empty}</p>;
  }
  return crews.map((crew) => {
    const mine = bots.filter((bot) => bot.crewId === crew.id);
    if (mine.length === 0) {
      return null;
    }
    return (
      <div key={crew.id} className="flex flex-col gap-1">
        <p className="px-1 text-muted text-xs">{crew.name}</p>
        <ul aria-label={crew.name} className="flex flex-col gap-0.5 rounded-xl bg-bubble p-1.5">
          {mine.map((bot) => (
            <li key={bot.id} className="flex items-center gap-2.5 rounded-lg px-2 py-1.5">
              <BotAvatar color={bot.color} size={24} still />
              <span className="min-w-0 flex-1 break-words text-sm">{bot.name}</span>
              <Switch
                checked={uses(bot)}
                disabled={locked}
                label={label(bot)}
                onChange={(next) => onChange(bot, next)}
              />
            </li>
          ))}
        </ul>
      </div>
    );
  });
}
