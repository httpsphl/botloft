// Dev only: the island over a fake desktop (`pnpm dev`, then
// /island.html), with a fake daemon where two agents wait and one works.

import { FakeBotloft } from "../lib/fake";
import { FakeIslandHost } from "./fakeHost";

export function islandPreview(): { api: FakeBotloft; host: FakeIslandHost } {
  document.documentElement.dataset.islandPreview = "";
  const fake = new FakeBotloft();
  const research = fake.addCrew("Research");
  const scout = fake.addBot(research.id, "Scout", "Finds sources");
  const writer = fake.addBot(research.id, "Writer", "Writes the weekly report");
  const review = fake.addBot(research.id, "Revisão", "Checks facts and tone");
  fake.setBotState(writer.id, "busy");
  fake.chat.ask(
    scout.id,
    "Bash",
    "curl",
    JSON.stringify({
      command: "curl -sL https://example.org/papers/week-40.zip -o inbox/week-40.zip",
    }),
    "Baixa os três artigos novos da pasta de entrada para ler hoje.",
  );
  fake.questions.ask(review.id, "Publico o relatório hoje ou espero a revisão do gráfico?", [
    "Hoje",
    "Esperar",
  ]);
  fake.chat.ask(
    scout.id,
    "Bash",
    "echo",
    `{"command": "python - <<'EOF'\\nimport re\\n${"x".repeat(60)}…`,
    "Confere os links do relatório.",
  );
  // The plan, past the warning line, and two routines coming up.
  const now = Date.now();
  fake.system = {
    ...fake.system,
    usage: {
      status: "allowed_warning",
      resetsAt: now + 2 * 3600_000,
      observedAt: now - 60_000,
      windows: [
        { name: "five_hour", utilization: 0.84, resetsAt: now + 2 * 3600_000 },
        { name: "seven_day", utilization: 0.46, resetsAt: now + 3 * 86_400_000 },
      ],
    },
  };
  const routine = (botId: string, name: string, minutes: number) =>
    fake.call("routines.create", {
      botId,
      name,
      prompt: name,
      schedule: { kind: "interval", minutes },
      timezone: "America/Sao_Paulo",
    });
  void routine(scout.id, "Novos artigos", 25);
  void routine(writer.id, "Resumo da semana", 300);
  // Earlier today: an edit and a turn, for the Today card.
  fake.chat.tool(writer.id, "Write", {
    summary: "week-40-outline.md",
    status: "done",
    file: "C:\\Work\\week-40-outline.md",
    lines: { added: 42, removed: 0 },
  });
  fake.chat.turn(writer.id);
  // Now: the writer edits the report, the reviewer runs a check.
  fake.chat.tool(writer.id, "Read", { summary: "week-40.md", status: "done" });
  fake.chat.tool(writer.id, "Edit", {
    summary: "week-40-report.md",
    status: "running",
    file: "C:\\Work\\week-40-report.md",
    input: JSON.stringify({
      file_path: "C:\\Work\\week-40-report.md",
      old_string: "## Destaques\n\nTODO: escrever os destaques",
      new_string:
        "## Destaques\n\n1. Busca híbrida ganha 7 pontos em textos longos.\n2. Escrever o plano antes corta um terço das chamadas.\n3. Testes fixos mudam menos que os sorteados.",
    }),
    lines: { added: 3, removed: 1 },
  });
  fake.setBotState(review.id, "busy");
  fake.chat.tool(review.id, "Bash", {
    summary: "npm run check-links",
    status: "running",
    input: JSON.stringify({ command: "npm run check-links" }),
  });
  // In a few seconds the reviewer finishes, and the island cheers.
  setTimeout(() => {
    fake.chat.reply(review.id, "Todos os 18 links do relatório funcionam.");
    fake.setBotState(review.id, "idle");
  }, 6000);
  const host = new FakeIslandHost();
  return { api: fake, host };
}
