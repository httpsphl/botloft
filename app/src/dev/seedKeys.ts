// Dev only: an agent's keys (spec 32), one kept and one that came from a
// backup without its value, and the agent asking for another.

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

export function seedKeys(fake: FakeBotloft, analyst: BotId): void {
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
}
