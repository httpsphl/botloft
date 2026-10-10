// Dev only: a crew led by its chief, who asks the owner for a new bot.

import type { FakeBotloft } from "../lib/fake";
import { SUGGEST_TOOL } from "../lib/fakeChat";

const PLAN = `Here's how I'd split it:

1. **Pages and copy**: I write the menu, hours and "about us" text myself.
2. **Look and feel**: this needs a designer. The crew has none, so I'm suggesting one.
3. **Build**: once the design is ready, @writer and I turn it into the site in the work folder.

I'll hand out the first tasks as soon as you approve the designer.`;

const DESIGNER = {
  name: "Designer",
  role: "Designs the pages: layout, colors and photos",
  instructions:
    "Design the bakery's site: home, menu, hours and contact. Put the mockups in design/ in the work folder, one image per page, and tell @chief when each is ready. Keep it warm and simple; the owner likes the colors of the shop's sign.",
  model: "sonnet",
  reason: "Nobody in the crew designs; the site needs a look before anyone builds it.",
};

export function seedChief(fake: FakeBotloft): void {
  void fake
    .call("crews.create", {
      name: "Bakery site",
      workFolder: "D:\\Projects\\Bakery site",
      lead: {
        name: "Chief",
        role: "Leads the crew: plans the work, suggests new agents and hands out tasks",
        instructions: "Build and keep up the website of my bakery.",
      },
    })
    .then((crew) => {
      const chief = crew.leadBotId;
      if (!chief) {
        return;
      }
      const writer = fake.addBot(crew.id, "Writer", "Writes the pages' text");
      fake.setBotState(writer.id, "idle", 1);
      fake.bot(chief).modelInUse = "claude-opus-5-5";
      const { chat } = fake;
      void fake.call("messages.send", {
        botId: chief,
        body: "Let's build the website for my bakery. What do you need?",
      });
      chat.finish(
        chat.tool(chief, "mcp__botloft__crew_roster", {}),
        '{"bots":[{"handle":"writer"}]}',
      );
      chat.reply(chief, PLAN);
      const input = JSON.stringify(DESIGNER);
      chat.tool(chief, SUGGEST_TOOL, { summary: "Designer", input });
      chat.ask(chief, SUGGEST_TOOL, "Designer", input);
      fake.setBotState(chief, "needs_approval", 1);
    });
}
