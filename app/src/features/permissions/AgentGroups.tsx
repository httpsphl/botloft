// A tab of the Permissions page (spec 15.1): what the owner let each agent
// do, one group per agent with its mascot, name and crew, or a line that
// says nothing is there yet.

import { Fragment, type ReactNode } from "react";
import type { Bot, BotId, Crew, CrewId } from "../../lib/protocol.gen";
import { BotAvatar } from "../bots/BotAvatar";

export interface Group<T> {
  bot: Bot;
  crew: string;
  items: T[];
}

/** `items` by agent, one agent only when `only` is set, crews then agents by name. */
export function groupByAgent<T extends { botId: BotId }>(
  items: T[],
  bots: Record<BotId, Bot>,
  crews: Record<CrewId, Crew>,
  only: BotId | null,
): Group<T>[] {
  const groups = new Map<BotId, Group<T>>();
  for (const item of items) {
    const bot = bots[item.botId];
    if (!bot || (only !== null && item.botId !== only)) {
      continue;
    }
    const group = groups.get(bot.id) ?? { bot, crew: crews[bot.crewId]?.name ?? "", items: [] };
    group.items.push(item);
    groups.set(bot.id, group);
  }
  return [...groups.values()].sort(
    (a, b) => a.crew.localeCompare(b.crew) || a.bot.name.localeCompare(b.bot.name),
  );
}

export const counted = (groups: Group<unknown>[]) =>
  groups.reduce((sum, group) => sum + group.items.length, 0);

export function AgentGroups<T>({
  groups,
  empty,
  keyOf,
  row,
}: {
  groups: Group<T>[];
  /** What the tab says with nothing in it. */
  empty: string;
  keyOf(item: T): string;
  row(bot: Bot, item: T): ReactNode;
}) {
  if (groups.length === 0) {
    return <p className="text-ink-soft text-sm leading-relaxed">{empty}</p>;
  }
  return (
    <div className="flex flex-col gap-5">
      {groups.map(({ bot, crew, items }) => (
        <section key={bot.id} aria-label={bot.name} className="flex flex-col gap-2">
          <h3 className="flex min-w-0 items-center gap-2 font-medium text-sm">
            <BotAvatar color={bot.color} size={20} still botId={bot.id} />
            <span className="truncate">{bot.name}</span>
            {crew && <span className="truncate text-muted text-xs">{crew}</span>}
          </h3>
          <ul className="flex flex-col gap-1.5">
            {items.map((item) => (
              <Fragment key={keyOf(item)}>{row(bot, item)}</Fragment>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
