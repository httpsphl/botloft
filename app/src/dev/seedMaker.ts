// Dev only: the Maker (spec 36), asked by the owner for crews, with a plan
// waiting in its chat.

import type { FakeBotloft } from "../lib/fake";

const ASKED =
  "I run a small bakery. Every week I write a post for the blog by hand and answer the same questions on WhatsApp about hours and orders.";

const PLAN = {
  title: "Two crews for the bakery",
  why: "You write a post every week by hand, and the same questions come in every day.",
  steps: [
    {
      op: "create_crew",
      ref: "blog",
      name: "Bakery blog",
      goal: "Publish one good post a week about the bakery: new breads, the season, the people.",
      lead_name: "Chief",
      lead_role: "Leads the crew: plans the work, suggests new agents and hands out tasks",
      memory: "A family bakery. Warm, simple tone. Never mention prices.",
    },
    {
      op: "add_agent",
      crew: "blog",
      name: "Writer",
      role: "Writes the posts",
      instructions: "Write each week's post in the work folder.",
      model: "sonnet",
    },
    {
      op: "add_routine",
      crew: "blog",
      agent: "Writer",
      name: "Weekly post",
      prompt: "Write this week's post and tell the chief where it is.",
      schedule: { kind: "weekly", days: [1], time: "08:00" },
      timezone: "America/Sao_Paulo",
    },
    {
      op: "create_crew",
      ref: "desk",
      name: "Customer desk",
      goal: "Draft answers to the customers' questions for the owner to send.",
      lead_name: "Chief",
      lead_role: "Leads the crew: plans the work, suggests new agents and hands out tasks",
    },
    {
      op: "add_agent",
      crew: "desk",
      name: "Support",
      role: "Drafts answers about hours and orders",
      instructions: "Draft short answers; never promise a delivery.",
      model: "haiku",
    },
    {
      op: "change_agent",
      crew: "Research",
      agent: "writer",
      model: "haiku",
      instructions:
        "Write the weekly report in at most one page, with the three main points first.",
      why: "You said the reports are too long to read.",
      before: {
        name: "Writer",
        role: "Turns notes into a weekly report",
        instructions: "Turn the crew's notes into a weekly report.",
        model: "sonnet",
        effort: "default",
      },
    },
    {
      op: "archive_agent",
      crew: "Research",
      agent: "planner",
      why: "It has not worked in a month.",
      before: { name: "Planner", role: "Plans changes before touching the code" },
    },
  ],
};

export function seedMaker(fake: FakeBotloft): void {
  const maker = fake.maker.ensure();
  fake.setBotState(maker.id, "needs_approval", 1);
  void fake.call("messages.send", { botId: maker.id, body: ASKED });
  fake.chat.reply(
    maker.id,
    "Here is what I would build. Leave out anything you do not want, then build it.",
  );
  fake.chat.ask(maker.id, "mcp__botloft__build", PLAN.title, JSON.stringify(PLAN));
}
