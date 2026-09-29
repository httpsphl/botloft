import type { Messages } from "../en";

export const crews: Messages["crews"] = {
  newCrew: "Nova equipe",
  newBot: "Novo bot",
  rename: "Renomear",
  paused: "Pausada",
  sidebar: {
    label: "Equipes",
    noMessages: "Nenhuma mensagem ainda",
    fromOwner: (text: string) => `Você: ${text}`,
    awaitingApproval: (text: string) => `Aguardando aprovação: ${text}`,
  },
  dialog: {
    renameTitle: "Renomear equipe",
    create: "Criar equipe",
    name: "Nome",
    namePlaceholder: "Pesquisa",
    createHint:
      "Os bots de uma equipe podem mandar mensagens uns aos outros e compartilham uma pasta.",
    renameHint: (slug: string) => `A pasta mantém o nome (${slug}).`,
  },
  view: {
    bots: (count: number) => (count === 1 ? "1 bot" : `${count} bots`),
    pausedNote: "pausada: os bots dela ficam parados até você retomá-la",
    pause: "Pausar equipe",
    resume: "Retomar equipe",
    moreActions: "Mais ações da equipe",
    openShared: "Abrir pasta compartilhada",
    archive: "Arquivar equipe",
    tabs: {
      label: "Visões da equipe",
      bots: "Bots",
      timeline: "Linha do tempo",
      tasks: "Tarefas",
    },
    timelineEmpty:
      "Nenhuma mensagem ainda. Escreva para um bot abaixo; o que os bots mandam uns aos outros também aparece aqui.",
    archiveTitle: (crew: string) => `Arquivar ${crew}?`,
    archiveBody: (bots: number) =>
      bots === 0
        ? "A equipe sai do app."
        : `A equipe sai do app. ${bots === 1 ? "O bot dela para" : `Os ${bots} bots dela param`}, e as mensagens que ainda esperam por ${bots === 1 ? "ele" : "eles"} não são entregues.`,
    failed: {
      pause: "Não foi possível pausar a equipe",
      resume: "Não foi possível retomar a equipe",
      openFolder: "Não foi possível abrir a pasta",
      archive: "Não foi possível arquivar a equipe",
    },
  },
  bots: {
    empty: (crew: string) => `Nenhum bot em ${crew} ainda.`,
    emptyHint:
      "Um bot é uma sessão do Claude Code que fica sempre rodando, com pasta e função próprias.",
    noRole: "Sem função ainda.",
  },
  tasks: {
    show: "Mostrar",
    open: "Abertas",
    all: "Todas",
    list: "Tarefas",
    noOpen: "Nenhuma tarefa aberta. Os bots criam tarefas uns para os outros com send_message.",
    none: "Nenhuma tarefa ainda. Os bots criam tarefas uns para os outros com send_message.",
    status: {
      open: "Aberta",
      done: "Concluída",
      failed: "Falhou",
      cancelled: "Cancelada",
      expired: "Vencida",
    },
    archivedBot: "um bot arquivado",
    asked: "pediu a",
    hop: (hops: number) => `etapa ${hops}`,
    hopHint: "Posição numa cadeia de delegações",
    due: (time: string) => `vence ${time}`,
    overdue: (time: string) => `atrasada, venceu ${time}`,
    ended: (status: string, time: string) => `${status.toLowerCase()} ${time}`,
  },
};
