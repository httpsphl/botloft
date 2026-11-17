import type { plugins as en } from "../en/plugins";

export const plugins: typeof en = {
  title: "Plugins",
  intro:
    "Os plugins ligados no seu próprio Claude Code, instalados nele ou vindos da sua conta do claude.ai. Um plugin junta skills, comandos e ajudantes. Escolha quais agentes usam cada um: o agente recebe uma cópia na pasta dele. Só agentes do Claude Code.",
  none: "Nenhum plugin encontrado. Os plugins que você instala e liga no Claude Code aparecem aqui.",
  source: {
    claude_code: "Claude Code",
    claude_ai: "claude.ai",
    kept: "Guardado pelo Botloft",
  },
  kept: "O plugin não está mais no seu Claude Code. O Botloft guarda a cópia dele para os agentes que o têm.",
  version: (version) => `Versão ${version}`,
  skills: (count) => (count === 0 ? "Nenhuma skill" : count === 1 ? "1 skill" : `${count} skills`),
  agents: (count) =>
    count === 0 ? "Nenhum agente ainda" : count === 1 ? "1 agente" : `${count} agentes`,
  noDescription: "Sem descrição.",
  brings: {
    title: "O que traz",
    skills: "Skills e comandos",
    agents: "Ajudantes que o agente pode chamar",
    nothing: "Nada que o Botloft consiga ver.",
  },
  warnings: {
    title: "Antes de dar",
    runs: "Ele roda comandos sozinho, sem perguntar a você, em momentos da conversa.",
    detail: (list) => `Em detalhe: ${list}`,
    programs:
      "Ele traz programas que o agente pode rodar no terminal. Cada um ainda pede a sua aprovação, como qualquer comando.",
    tools: (names) =>
      `As ferramentas conectadas dele não vêm junto: ${names}. Para dar uma ferramenta a um agente, use Ferramentas.`,
    settings:
      "Ele pede configurações que o Botloft não passa, então partes dele podem não funcionar.",
  },
  who: "Quem pode usar",
  canUse: (bot) => `${bot} pode usar`,
  noAgents: "Nenhum agente do Claude Code ainda.",
  shareFailed: "Não deu para mudar quem usa este plugin.",
  search: "Buscar plugins",
  nothingFound: (query) => `Nenhum plugin com "${query}".`,
  everyone: {
    all: "Todos os plugins para todos os agentes",
    allHint:
      "Os agentes do Claude Code usam todos os seus plugins, inclusive os que você ligar depois, e os agentes que você criar depois também.",
    allAsk: (count) =>
      count === 1
        ? "Dar o plugin a todos os agentes?"
        : `Dar os ${count} plugins a todos os agentes?`,
    allBody:
      "Cada agente do Claude Code recebe uma cópia de cada um e recomeça quando terminar o que está fazendo, na mesma conversa. Cada agente lê um resumo das skills de todos os plugins em toda conversa, o que gasta um pouco mais do seu plano.",
    allRisky: (count) =>
      count === 1
        ? "1 deles roda comandos sozinho, sem perguntar a você."
        : `${count} deles rodam comandos sozinhos, sem perguntar a você.`,
    allGive: "Dar a todos",
    one: "Todos os agentes",
    oneHint: "Inclusive os agentes que você criar depois.",
    byAll: "Todos os plugins estão ligados para todos, no alto de Plugins.",
    failed: "Não deu para mudar quem usa os seus plugins.",
  },
};
