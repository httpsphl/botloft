// O histórico da memória e das instruções de um agente (spec 8.10).

import type { versions as en } from "../en/versions";

export const versions: typeof en = {
  memoryHistory: "Histórico da memória",
  instructionsHistory: "Histórico das instruções",
  memory: "Memória",
  memoryAbout: (bot) =>
    `O que ${bot} guarda de uma conversa para outra, no CLAUDE.md da pasta dele. ${bot} escreve; toda mudança fica guardada.`,
  title: {
    memory: (bot) => `Histórico da memória de ${bot}`,
    instructions: (bot) => `Histórico das instruções de ${bot}`,
  },
  intro: {
    memory: (bot) =>
      `Cada mudança na memória de ${bot}, da mais nova para a mais antiga. O Botloft guarda este histórico fora da pasta de ${bot}, então ${bot} não consegue mudá-lo.`,
    instructions: (bot) =>
      `Cada mudança nas instruções de ${bot}, da mais nova para a mais antiga.`,
  },
  empty: {
    memory: (bot) =>
      `Nenhuma mudança ainda. O histórico começa na primeira vez que ${bot} ou outra pessoa mudar a memória.`,
    instructions:
      "Nenhuma mudança ainda. O histórico começa na primeira vez que as instruções mudarem.",
  },
  list: "Versões",
  author: {
    start: "Como estava no começo",
    owner: "Você mudou",
    restored: (at) => `Você voltou à versão de ${at}`,
    agent: (bot) => `${bot} mudou`,
    maker: "O Criador mudou",
    outside: "Mudou fora do Botloft",
  },
  now: "Agora",
  emptyText: "(vazia)",
  showChanges: "Ver as mudanças",
  restore: "Voltar a esta versão",
  restoreConfirm: {
    title: "Voltar a esta versão?",
    memory: (bot, at) =>
      `A memória de ${bot} volta a como estava em ${at}. A versão de agora fica no histórico, então dá para desfazer isto também. ${bot} lê a memória na próxima vez que começar uma conversa.`,
    instructions: (bot, at) =>
      `As instruções de ${bot} voltam a como estavam em ${at}. A versão de agora fica no histórico, então dá para desfazer isto também.`,
    confirm: "Voltar",
  },
  failed: {
    load: "Não deu para carregar o histórico",
    restore: "Não deu para voltar àquela versão",
  },
};
