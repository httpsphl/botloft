import type { skills as en } from "../en/skills";

export const skills: typeof en = {
  title: "Skills",
  intro:
    "As skills do seu próprio Claude Code, da sua pasta de skills e da sua conta do claude.ai. Escolha quais agentes usam cada uma: o agente recebe uma cópia na pasta dele. Só agentes do Claude Code.",
  none: "Nenhuma skill encontrada. Uma skill é uma pasta com um SKILL.md em .claude\\skills, na sua pasta de usuário.",
  failed: "Não deu para ler as suas skills.",
  source: {
    personal: "Sua",
    claude_ai: "claude.ai",
    kept: "Guardada pelo Botloft",
  },
  kept: "O original sumiu. O Botloft guarda a cópia dele para os agentes que a têm.",
  agents: (count) =>
    count === 0 ? "Nenhum agente ainda" : count === 1 ? "1 agente" : `${count} agentes`,
  noDescription: "Sem descrição.",
  warnings: {
    title: "Antes de dar",
    tools: (tools) => `Enquanto é usada, ela roda isto sem perguntar a você: ${tools}`,
    commands: "Ela roda comandos sozinha quando é usada.",
    files: "Ela traz outros arquivos, como scripts.",
  },
  who: "Quem pode usar",
  canUse: (bot) => `${bot} pode usar`,
  noAgents: "Nenhum agente do Claude Code ainda.",
  shareFailed: "Não deu para mudar quem usa esta skill.",
  search: "Buscar skills",
  nothingFound: (query) => `Nenhuma skill com "${query}".`,
  everyone: {
    all: "Todas as skills para todos os agentes",
    allHint:
      "Os agentes do Claude Code usam todas as suas skills, inclusive as que você adicionar depois, e os agentes que você criar depois também.",
    allAsk: (count) => `Dar as ${count} skills a todos os agentes?`,
    allBody:
      "Cada agente do Claude Code recebe uma cópia de cada uma e recomeça quando terminar o que está fazendo, na mesma conversa. Cada agente lê um resumo de todas as skills em toda conversa, o que gasta um pouco mais do seu plano.",
    allRisky: (count) =>
      count === 1
        ? "1 delas roda ferramentas sem perguntar a você, ou comandos sozinha."
        : `${count} delas rodam ferramentas sem perguntar a você, ou comandos sozinhas.`,
    allGive: "Dar a todos",
    one: "Todos os agentes",
    oneHint: "Inclusive os agentes que você criar depois.",
    byAll: "Todas as skills estão ligadas para todos, no alto de Skills.",
    failed: "Não deu para mudar quem usa as suas skills.",
    allTooBig: (count) =>
      count === 1
        ? "1 é grande demais para dar e fica de fora."
        : `${count} são grandes demais para dar e ficam de fora.`,
  },
  tooBig: {
    card: "Grande demais para dar",
    title: "Grande demais para dar aos agentes",
    body: "Cada agente recebe a própria cópia de uma skill, de até 2.000 arquivos e 50 MB, e esta passa disso. Uma pasta de pacotes baixados dentro dela, como node_modules, costuma ser o motivo.",
  },
  request: {
    asks: (bot, name) => `${bot} pede a skill ${name}`,
    wants: (bot, name) => `${bot} quer usar a sua skill ${name}`,
    give: "Dar a skill",
    given: (name, bot) => `Você deu ${name} a ${bot}`,
    declined: (name) => `Você não deu ${name}`,
    expired: (name) => `O pedido de ${name} expirou`,
  },
};
