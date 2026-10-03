// Dev only: questions the bots left for the owner overnight (spec 23).

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

const HOUR = 60 * 60 * 1000;

export function seedQuestions(fake: FakeBotloft, crew: { scout: BotId; analyst: BotId }): void {
  const now = Date.now();
  fake.now = now - 7 * HOUR;
  fake.chat.tool(crew.analyst, "mcp__botloft__ask_owner", { status: "done" });
  fake.questions.ask(
    crew.analyst,
    "Two sources disagree on last year's **flour prices** (12% and 19% up). Which one should the report trust?\n\nThe 19% figure comes from a smaller survey.",
  );
  fake.now = now - 20 * 60 * 1000;
  fake.chat.tool(crew.scout, "mcp__botloft__ask_owner", { status: "done" });
  fake.questions.ask(
    crew.scout,
    "Should next week's summary cover only papers, or blog posts too?",
    ["Only papers", "Papers and blog posts", "Skip next week"],
  );
  fake.now = now;
}
