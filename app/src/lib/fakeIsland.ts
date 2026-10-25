// The fake daemon's island methods (spec 31): the cards that wait, built
// from the fake chat and questions with the phone's rules (28.5), and what
// the crews did since a moment, counted like the daemon's `today.rs`.

import type { FakeBotloft, Handlers } from "./fake";
import type { ApprovalCard, BusyBot, QuestionCard } from "./protocol.gen";

/** The longest text a card shows whole (the daemon's `CARD_TEXT_MAX`). */
const CARD_TEXT_MAX = 8 * 1024;

/** The tools a card may allow; the rest is answered in the Botloft. */
const MAY_ALLOW = new Set([
  "Bash",
  "PowerShell",
  "WebFetch",
  "WebSearch",
  "Read",
  "Write",
  "Edit",
  "MultiEdit",
  "NotebookEdit",
  "ExitPlanMode",
  "mcp__botloft__show_plan",
]);

/** What to read before allowing, and whether it is not all of the request. */
function textOf(input: string): { text: string; cut: boolean } {
  let parsed: unknown;
  try {
    parsed = JSON.parse(input);
  } catch {
    return { text: input.slice(0, CARD_TEXT_MAX), cut: true };
  }
  const fields = parsed as { command?: unknown; plan?: unknown };
  const text =
    typeof fields.command === "string"
      ? fields.command
      : typeof fields.plan === "string"
        ? fields.plan
        : input;
  return text.length > CARD_TEXT_MAX
    ? { text: text.slice(0, CARD_TEXT_MAX), cut: true }
    : { text, cut: false };
}

export function islandHandlers(fake: FakeBotloft): Pick<Handlers, "cards.list" | "activity.today"> {
  const who = (botId: string) => {
    const bot = fake.bot(botId, false);
    return { bot: { name: bot.name, color: bot.color }, crew: fake.crew(bot.crewId, false).name };
  };
  return {
    "cards.list": () => {
      const approvals: ApprovalCard[] = [];
      for (const item of fake.chat.items) {
        if (item.body.kind !== "approval" || item.body.status !== "pending") {
          continue;
        }
        approvals.push({
          approvalId: item.body.approvalId,
          ...who(item.botId),
          createdAt: item.createdAt,
          toolName: item.body.toolName,
          summary: item.body.summary,
          ...(item.body.explanation !== null && { explanation: item.body.explanation }),
          ...textOf(item.body.input),
          atComputer: !MAY_ALLOW.has(item.body.toolName),
        });
      }
      const questions: QuestionCard[] = fake.questions.asked
        .map((entry) => entry.question)
        .filter((question) => question.status === "open")
        .map((question) => ({
          questionId: question.id,
          ...who(question.botId),
          createdAt: question.createdAt,
          text: question.text,
          options: question.options,
        }));
      const oldest = (a: { createdAt: number }, b: { createdAt: number }) =>
        a.createdAt - b.createdAt;
      return { approvals: approvals.sort(oldest), questions: questions.sort(oldest) };
    },
    "activity.today": ({ since }) => {
      const tasks = [...fake.conversation.tasks.values()].filter((task) => task.updatedAt >= since);
      const files = new Set<string>();
      const lines = { added: 0, removed: 0 };
      let requestsAnswered = 0;
      const work = new Map<string, number>();
      for (const item of fake.chat.items) {
        const body = item.body;
        if (
          body.kind === "tool" &&
          item.createdAt >= since &&
          body.status === "done" &&
          body.lines
        ) {
          if (body.file) files.add(body.file);
          lines.added += body.lines.added;
          lines.removed += body.lines.removed;
        }
        if (body.kind === "approval" && item.updatedAt >= since) {
          if (body.status === "allowed" || body.status === "denied") requestsAnswered += 1;
        }
        if (body.kind === "turn" && item.createdAt >= since) {
          work.set(item.botId, (work.get(item.botId) ?? 0) + body.durationMs);
        }
      }
      const [top] = [...work.entries()].filter(([, ms]) => ms > 0).sort((a, b) => b[1] - a[1]);
      let busiest: BusyBot | null = null;
      if (top) {
        const bot = fake.bot(top[0], false);
        busiest = {
          botId: bot.id,
          name: bot.name,
          color: bot.color,
          crew: fake.crew(bot.crewId, false).name,
          workMs: top[1],
        };
      }
      return {
        tasksDone: tasks.filter((task) => task.status === "done").length,
        tasksFailed: tasks.filter((task) => task.status === "failed" || task.status === "expired")
          .length,
        botMessages: fake.conversation.messages.filter(
          (message) => message.fromKind === "bot" && message.createdAt >= since,
        ).length,
        requestsAnswered,
        filesEdited: files.size,
        lines,
        busiest,
      };
    },
  };
}
