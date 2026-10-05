// The fake daemon's tokens per bot (spec 8.7), summed from the `turn` items
// of the fake chats, as the daemon sums its stored ones. The share of the
// plan is the tokens times `planSharePerToken`, once a test sets it.

import type { FakeBotloft, Handlers } from "./fake";
import { usedTokens } from "./format";
import type { BotId, BotTokens } from "./protocol.gen";

export function usageHandlers(fake: FakeBotloft): Pick<Handlers, "usage.tokens"> {
  return {
    "usage.tokens": ({ since }) => {
      const byBot = new Map<BotId, BotTokens>();
      for (const item of fake.chat.items) {
        if (item.body.kind !== "turn" || !item.body.tokens || item.createdAt < since) {
          continue;
        }
        let entry = byBot.get(item.botId);
        if (!entry) {
          const bot = fake.bot(item.botId, false);
          entry = {
            botId: bot.id,
            name: bot.name,
            color: bot.color,
            crew: fake.crew(bot.crewId, false).name,
            archived: bot.archivedAt !== null,
            turns: 0,
            tokens: { input: 0, cacheWrite: 0, reloaded: 0, cacheRead: 0, output: 0 },
            planShare: null,
          };
          byBot.set(bot.id, entry);
        }
        const { input, cacheWrite, reloaded, cacheRead, output } = item.body.tokens;
        entry.turns += 1;
        entry.tokens = {
          input: entry.tokens.input + input,
          cacheWrite: entry.tokens.cacheWrite + cacheWrite,
          reloaded: entry.tokens.reloaded + reloaded,
          cacheRead: entry.tokens.cacheRead + cacheRead,
          output: entry.tokens.output + output,
        };
      }
      const rate = fake.planSharePerToken;
      for (const entry of byBot.values()) {
        entry.planShare = rate === null ? null : usedTokens(entry.tokens) * rate;
      }
      return [...byBot.values()].sort(
        (a, b) =>
          usedTokens(b.tokens) - usedTokens(a.tokens) ||
          a.name.localeCompare(b.name) ||
          a.crew.localeCompare(b.crew),
      );
    },
  };
}
