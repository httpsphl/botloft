// Dev only: a bot that shows its plan before long work (spec 10.5).

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

export function seedPlan(fake: FakeBotloft, bot: BotId): void {
  fake.bot(bot).planFirst = true;
  void fake.call("messages.send", {
    botId: bot,
    body: "Rewrite the whole sourdough guide for beginners, with the new photos.",
  });
  fake.chat.tool(bot, "mcp__botloft__show_plan", { status: "done" });
  const plan = {
    summary: "A beginner's sourdough guide, rewritten from the start, with the new photos",
    steps: [
      "Read the current guide and the notes in the work folder",
      "Write a shorter guide in five parts, from starter to first loaf",
      "Put the new photos next to the steps they show",
      "Ask Revisão to check facts and tone",
    ],
    alternative: "Only fix the hardest part, feeding the starter, and leave the rest",
    // Long enough to look at it in the preview.
    starts_at: Date.now() + 90_000,
  };
  fake.chat.ask(bot, "mcp__botloft__show_plan", plan.summary, JSON.stringify(plan));
}
