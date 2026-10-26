// Dev only: an agent's keys (spec 32), one kept and one that came from a
// backup without its value, the agent asking for another, and keys pasted
// in the conversations.

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

export function seedKeys(fake: FakeBotloft, analyst: BotId, scout: BotId): void {
  const now = Date.now();
  fake.now = now - 3 * 24 * 60 * 60 * 1000;
  void fake.call("keys.set", {
    botId: analyst,
    name: "SHEETS_API_KEY",
    value: "dev-sheets-0000000000",
    description: "Google Sheets, read only",
  });
  void fake.call("keys.set", { botId: analyst, name: "STRIPE_KEY", value: "dev-stripe-000000" });
  const restored = fake.keys.saved.get(analyst)?.get("STRIPE_KEY");
  if (restored) {
    restored.missing = true;
  }
  fake.now = now - 5 * 60 * 1000;
  fake.chat.tool(analyst, "mcp__botloft__ask_key", { status: "done" });
  fake.keys.ask(
    analyst,
    "GITHUB_TOKEN",
    "To open the pull request with the new price table. A token with access to the bakery-site repository is enough: GitHub, Settings, Developer settings, Fine-grained tokens.",
  );
  fake.now = now;
  // Settings, Security: two keys pasted before they were kept apart.
  fake.keys.pasted.push(
    {
      id: "dev_github",
      value: "ghp_dev0000000000000000000000",
      preview: "ghp_…0000",
      botIds: [analyst, scout],
      places: 3,
      lastAt: now - 2 * 24 * 60 * 60 * 1000,
    },
    {
      id: "dev_openai",
      value: "sk-dev00000000000000000000",
      preview: "sk-…0000",
      botIds: [scout],
      places: 1,
      lastAt: now - 9 * 24 * 60 * 60 * 1000,
    },
  );
}
