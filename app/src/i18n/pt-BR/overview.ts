// A visão geral ao lado do chat do agente (spec 33).

import type { Messages } from "../en";

export const overview: Messages["overview"] = {
  heading: "Visão geral",
  panel: (bot) => `Visão geral de ${bot}`,
  show: "Mostrar visão geral",
  hide: "Esconder visão geral",
  close: "Fechar",
  context: {
    title: "Espaço da conversa",
    used: (percent) => `${percent} ocupado`,
    of: (used, total) => `${used} de ${total}`,
    bar: (percent) => `Espaço da conversa, ${percent} ocupado`,
    unknown: (bot) => `${bot} ainda não disse quanto da conversa está ocupado.`,
    noParts: "Este motor só dá o total, não o que ocupa.",
    parts: "O que ocupa",
    free: "Livre",
    kind: {
      messages: "Conversa",
      instructions: "Instruções do Claude Code",
      tools: "Ferramentas do Claude Code",
      connected_tools: "Ferramentas conectadas",
      memory: "Memória",
      skills: "Habilidades",
      agents: "Subagentes",
      buffer: "Reserva para compactar",
      other: "Outros",
    },
    memory: "Arquivos de memória",
    memoryHint: (bot) => `O que ${bot} lê no começo de toda conversa.`,
  },
};
