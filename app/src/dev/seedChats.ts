// Dev only: a morning's work between five research bots, for the preview.

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

/** A small bar chart, as a PNG the owner "attached". */
const CHART_PNG =
  "iVBORw0KGgoAAAANSUhEUgAAAGAAAABACAIAAABqVuVZAAAAmElEQVR42u3bQQmAMBiA0RWRtRFb7OxpEQxjFUvYQlgBG4z/MBDZgy/Bu3+ptUedEgJAgAABAvRFZ0nBAAECBAgQIECARpfLFQwQIECAAAECBAgQIECAAAECNAjo3nIwQIAAAQIECBAgQOOAlmMNBggQIECAAAECBAgQIECAAAECBAgQIECAAAGaEajWfdpcz65nQIAA/aoXK4mdiO7oCAwAAAAASUVORK5CYII=";

const SUMMARY = `Done. The summary is in \`shared/summaries/week-39.md\`.

**Highlights**

1. *Retrieval at scale*: hybrid search beats dense-only by 7 points on long documents.
2. *Agents that plan*: writing the plan down first cut tool calls by a third.
3. *Small evals*: fixed test sets drift less than sampled ones.

| Paper | Pages | Worth a deep read |
|---|---|---|
| Retrieval at scale | 14 | Yes |
| Agents that plan | 22 | Yes |
| Small evals | 9 | Maybe |

Your chart matches the numbers in the first paper. I asked @writer to turn this into the weekly report.`;

const PLAN = `## Cache the weekly report

The report is rebuilt on every visit, and Monday is when everyone opens it.

1. Build the report once when the week closes and save it in \`shared/cache/\`.
2. Serve the saved copy; rebuild it only when a source file changes.
3. Add a test that the saved copy matches a fresh build.

**Not changing:** the report's layout or its sources.`;

interface Crew {
  scout: BotId;
  writer: BotId;
  reviewer: BotId;
  analyst: BotId;
  planner: BotId;
}

const MINUTE = 60 * 1000;

export function seedChats(fake: FakeBotloft, crew: Crew): void {
  const { chat, conversation: talk } = fake;
  const end = Date.now();
  const at = (minutesAgo: number) => {
    fake.now = end - minutesAgo * MINUTE;
  };

  at(95);
  void fake.call("messages.send", {
    botId: crew.scout,
    body: "Summarize the three newest papers in shared/inbox. Here's last week's chart for reference.",
    attachments: [{ name: "chart-week-38.png", mediaType: "image/png", data: CHART_PNG }],
  });
  const asked = fake.conversation.messages.at(-1);
  if (asked) {
    talk.read(talk.deliveryOf(asked.id).id);
  }
  at(94);
  const files = [
    ["Glob", "shared/inbox/*.pdf"],
    ["Read", "2026-09-27-retrieval.pdf"],
    ["Read", "2026-09-26-agents.pdf"],
    ["Read", "2026-09-25-evals.pdf"],
    ["Write", "week-39.md"],
  ] as const;
  for (const [name, summary] of files) {
    const file = name === "Write" ? "C:\\Work\\summaries\\week-39.md" : null;
    const call = chat.tool(crew.scout, name, {
      summary,
      input: JSON.stringify({ path: summary }),
      file,
    });
    chat.finish(call, name === "Write" ? "Wrote 42 lines" : "ok");
  }
  at(93);
  const made = fake.files;
  made.add(crew.scout, "week-39.md", {
    folder: "summaries",
    text: SUMMARY,
    writtenByBot: true,
    at: fake.now,
  });
  made.add(crew.scout, "papers.csv", {
    text: "paper,pages,deep_read\nRetrieval at scale,14,yes\nAgents that plan,22,yes\nSmall evals,9,maybe\n",
    at: fake.now + MINUTE,
  });
  made.add(crew.scout, "week-39-report.docx", { text: "docx", at: fake.now + 2 * MINUTE });
  at(92);
  chat.reply(crew.scout, SUMMARY);
  const handoff = chat.tool(crew.scout, "mcp__botloft__send_message", {
    summary: "to @writer",
    input: JSON.stringify({ to: "writer", kind: "task", body: "Turn week-39.md into the report" }),
  });
  chat.finish(handoff, '{"ok":true}');

  at(91);
  const task = talk.task(crew.scout, crew.writer, { hops: 1 });
  const note = talk.say({
    from: crew.scout,
    to: crew.writer,
    kind: "task",
    taskId: task.id,
    body: "Turn shared/summaries/week-39.md into this week's report draft.",
  });
  talk.read(note.delivery.id);
  chat.turn(crew.scout);
  at(90);
  chat.finish(chat.tool(crew.writer, "Read", { summary: "week-39.md" }), "42 lines");
  chat.reply(crew.writer, "Starting the draft. I'll keep the three highlights as sections.");
  chat.tool(crew.writer, "Write", { summary: "week-39-report.md" });

  at(40);
  const review = talk.task(crew.writer, crew.reviewer, { hops: 2 });
  const check = talk.say({
    from: crew.writer,
    to: crew.reviewer,
    kind: "task",
    taskId: review.id,
    body: "Check the numbers in section 2 against the sources.",
  });
  talk.read(check.delivery.id);
  at(39);
  chat.reply(crew.reviewer, "I'll run the link checker first, then compare each figure.");
  const command = JSON.stringify({
    command: "npm run check-links",
    description: "Check the links",
  });
  chat.tool(crew.reviewer, "Bash", { summary: "Check the links", input: command });
  chat.ask(crew.reviewer, "Bash", "npm run check-links", command);
  at(12);
  const late = talk.say({
    from: crew.scout,
    to: crew.reviewer,
    body: "Two of the links in the draft moved; the new ones are in shared/links.md.",
  });
  talk.deliver(late.delivery.id, "dead", "the bot did not start in time");

  at(30);
  void fake.call("messages.send", {
    botId: crew.analyst,
    body: "Can you check the growth numbers in section 2?",
  });
  at(29);
  chat.finish(chat.tool(crew.analyst, "Read", { summary: "week-39-report.md" }), "118 lines");
  chat.add(crew.analyst, {
    kind: "notice",
    level: "warning",
    text: "The account reached its usage limit. Messages wait until it resets.",
  });
  chat.turn(crew.analyst, "rate_limit");

  at(8);
  void fake.call("messages.send", {
    botId: crew.planner,
    body: "The report pages are slow on Mondays. Plan a fix, but don't change anything yet.",
  });
  at(7);
  chat.finish(chat.tool(crew.planner, "Grep", { summary: "buildReport" }), "3 matches");
  const plan = JSON.stringify({ plan: PLAN });
  chat.tool(crew.planner, "ExitPlanMode", { summary: "Cache the weekly report", input: plan });
  chat.ask(crew.planner, "ExitPlanMode", "Cache the weekly report", plan);
  fake.now = end;
}
