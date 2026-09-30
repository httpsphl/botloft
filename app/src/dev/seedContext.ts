// The preview's effort and conversation space (spec 7.4, 8.6): what Claude
// Code would have reported for each bot, and a compaction that ends a
// moment after it is asked for.

import type { FakeBotloft } from "../lib/fake";
import type { BotId, ContextUsage } from "../lib/protocol.gen";

const COMPACTION_MS = 2500;

const holds = (usedTokens: number, windowTokens: number): ContextUsage => ({
  usedTokens,
  windowTokens,
  autoCompactTokens: windowTokens - 33_000,
  compacting: false,
  updatedAt: Date.now(),
});

export function seedContext(
  fake: FakeBotloft,
  bots: { scout: BotId; writer: BotId; reviewer: BotId; analyst: BotId; planner: BotId },
): void {
  for (const id of [bots.scout, bots.writer, bots.reviewer, bots.planner]) {
    fake.bot(id).effortDefault = "medium";
  }
  // Haiku takes no effort level.
  fake.bot(bots.analyst).effortDefault = "none";
  fake.bot(bots.reviewer).effort = "high";
  fake.setContext(bots.scout, holds(556_000, 1_000_000));
  fake.setContext(bots.writer, holds(912_400, 1_000_000));
  fake.setContext(bots.reviewer, holds(61_800, 1_000_000));
  fake.setContext(bots.analyst, holds(24_300, 200_000));

  fake.subscribe((event) => {
    if (event.name !== "bot.context" || !event.params.context?.compacting) {
      return;
    }
    const { botId, context } = event.params;
    setTimeout(() => {
      fake.chat.add(botId, {
        kind: "notice",
        level: "info",
        code: "compacted",
        text: "The conversation was compacted: earlier messages are now a summary.",
      });
      fake.setContext(botId, holds(24_000, context.windowTokens));
    }, COMPACTION_MS);
  });
}
