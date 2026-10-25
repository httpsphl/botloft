// Dev only: the island over a fake desktop (`pnpm dev`, then
// /island.html), with a fake daemon where two agents wait and one works.

import { FakeBotloft } from "../lib/fake";
import type { IslandHost } from "./host";

export function islandPreview(): { api: FakeBotloft; host: IslandHost } {
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
  const host: IslandHost = {
    daemon: async () => ({ port: 0, token: "" }),
    show: async () => {},
    hide: async () => {},
    openApp: async (botId) => {
      console.info("open Botloft", botId ?? "");
    },
  };
  return { api: fake, host };
}
