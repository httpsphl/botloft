// The fake daemon's usage (spec 8.7): each bot's tokens summed from the
// `turn` items of the fake chats, and the cost from `costParts`, a ledger
// like the daemon's `turn_costs` that tests and the dev seed fill.

import type { FakeBotloft, Handlers } from "./fake";
import { usedTokens } from "./format";
import type { BotId, BotTokens, CostGroup, RoutineId } from "./protocol.gen";

/** One model's part of what a turn cost, at API prices in US dollars. */
export interface FakeCostPart {
  botId: BotId;
  at: number;
  cost: number;
  model: string | null;
  routineId: RoutineId | null;
}

/** `2026-11-14`, the local day of `at`, as the daemon writes it. */
function dayOf(at: number): string {
  const date = new Date(at);
  const two = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${two(date.getMonth() + 1)}-${two(date.getDate())}`;
}

export function usageHandlers(fake: FakeBotloft): Pick<Handlers, "usage.tokens" | "usage.costs"> {
  const parts = (since: number) => fake.costParts.filter((part) => part.at >= since);
  const entry = (byBot: Map<BotId, BotTokens>, botId: BotId): BotTokens => {
    let found = byBot.get(botId);
    if (!found) {
      const bot = fake.bot(botId, false);
      found = {
        botId: bot.id,
        name: bot.name,
        color: bot.color,
        crew: fake.crew(bot.crewId, false).name,
        archived: bot.archivedAt !== null,
        turns: 0,
        tokens: { input: 0, cacheWrite: 0, reloaded: 0, cacheRead: 0, output: 0 },
        // A bot on Claude always has a price; the other engines give none.
        cost: bot.agent === "claude" ? 0 : null,
      };
      byBot.set(botId, found);
    }
    return found;
  };
  return {
    "usage.tokens": ({ since }) => {
      const byBot = new Map<BotId, BotTokens>();
      for (const item of fake.chat.items) {
        if (item.body.kind !== "turn" || !item.body.tokens || item.createdAt < since) {
          continue;
        }
        const bot = entry(byBot, item.botId);
        const { input, cacheWrite, reloaded, cacheRead, output } = item.body.tokens;
        bot.turns += 1;
        bot.tokens = {
          input: bot.tokens.input + input,
          cacheWrite: bot.tokens.cacheWrite + cacheWrite,
          reloaded: bot.tokens.reloaded + reloaded,
          cacheRead: bot.tokens.cacheRead + cacheRead,
          output: bot.tokens.output + output,
        };
      }
      for (const part of parts(since)) {
        const bot = entry(byBot, part.botId);
        bot.cost = (bot.cost ?? 0) + part.cost;
      }
      return [...byBot.values()].sort(
        (a, b) =>
          Number(a.cost === null) - Number(b.cost === null) ||
          (b.cost ?? 0) - (a.cost ?? 0) ||
          usedTokens(b.tokens) - usedTokens(a.tokens) ||
          a.name.localeCompare(b.name) ||
          a.crew.localeCompare(b.crew),
      );
    },
    "usage.costs": ({ since, by }) => {
      const groups = new Map<string, CostGroup>();
      for (const part of parts(since)) {
        const key =
          by === "model"
            ? (part.model ?? "")
            : by === "day"
              ? dayOf(part.at)
              : (part.routineId ?? "");
        let group = groups.get(key);
        if (!group) {
          group = { key, cost: 0 };
          const routine =
            by === "routine" && part.routineId
              ? fake.routines.routines.get(part.routineId)
              : undefined;
          if (routine) {
            const bot = fake.bot(routine.botId, false);
            group = { ...group, name: routine.name, botName: bot.name, color: bot.color };
          }
          groups.set(key, group);
        }
        group.cost += part.cost;
      }
      const list = [...groups.values()];
      return by === "day"
        ? list.sort((a, b) => b.key.localeCompare(a.key))
        : list.sort((a, b) => b.cost - a.cost || a.key.localeCompare(b.key));
    },
  };
}
