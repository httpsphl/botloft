import type { Messages } from "../en";

export const crews: Messages["crews"] = {
  newCrew: "Nova equipe",
  newBot: "Novo bot",
  rename: "Renomear",
  paused: "Pausada",
  sidebar: {
    menuOf: (crew) => `Ações de ${crew}`,
    label: "Equipes",
    noMessages: "Nenhuma mensagem ainda",
    fromOwner: (text: string) => `Você: ${text}`,
    awaitingApproval: (text: string) => `Aguardando aprovação: ${text}`,
    unread: "não lida",
    unreadCount: (count: number) => (count === 1 ? "1 não lida" : `${count} não lidas`),
    showAll: "Ver todas as equipes",
    collapseAll: "Recolher todas as equipes",
    expandAll: "Expandir todas as equipes",
    collapse: (crew: string) => `Recolher ${crew}`,
    expand: (crew: string) => `Expandir ${crew}`,
    waiting: "um bot precisa de você",
  },
  overview: {
    count: (count: number) => (count === 1 ? "1 equipe" : `${count} equipes`),
    working: (count: number) => (count === 1 ? "1 trabalhando" : `${count} trabalhando`),
    waiting: (count: number) => (count === 1 ? "1 precisa de você" : `${count} precisam de você`),
    calm: "Tudo calmo",
    noBots: "Nenhum bot ainda",
  },
  dialog: {
    folder: "Pasta de trabalho",
    folderHint: "Onde os bots guardam o que fazem. Pode ser uma pasta que você já usa.",
    folderDefault: "Uma pasta nova dentro do Botloft",
    chooseFolder: "Escolher pasta…",
    pickTitle: "Escolha onde a equipe trabalha",
    useDefault: "Usar uma pasta nova",
    goal: "Para que é esta equipe?",
    goalPlaceholder: "Criar e manter o site da minha padaria",
    goalHint:
      "A equipe começa com um Chefe, que lê isto, planeja o trabalho e sugere os bots de que precisa.",
    chiefModel: "Modelo do Chefe",
    chiefName: "Chefe",
    chiefRole: "Lidera a equipe: planeja o trabalho, sugere bots novos e distribui as tarefas",
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
    folder: (path: string) => `Trabalha em ${path}`,
    openFolder: "Abrir pasta de trabalho",
    openFolderHint: "Clique para abrir",
    changeFolder: "Mudar pasta de trabalho…",
    moveTitle: (crew: string) => `Mudar ${crew} para outra pasta?`,
    moveBody: (path: string) =>
      `Os bots vão trabalhar em ${path}. Cada um reinicia quando terminar o que está fazendo. O que já foi feito fica onde está.`,
    move: "Mudar",
    archive: "Arquivar equipe",
    delete: "Excluir equipe",
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
    deleteTitle: (crew: string) => `Excluir ${crew}?`,
    deleteBody: (crew: string, bots: number, running: boolean) =>
      bots === 0
        ? `${crew} sai do Botloft de vez. Não dá para desfazer.`
        : `${crew} e ${bots === 1 ? "o bot dela" : `os ${bots} bots dela`} ${running ? "param agora e saem" : "saem"} do Botloft de vez, com as conversas, as rotinas e as tarefas. Não dá para desfazer.`,
    deleteKept:
      "As pastas continuam no seu computador, com tudo o que está nelas: a pasta de cada bot e a pasta de trabalho da equipe:",
    deleteRecycle: "Mandar as pastas da equipe para a Lixeira",
    deleteRecycled:
      "A pasta da equipe vai para a Lixeira, com a pasta de cada bot e a pasta de trabalho dentro. Ainda dá para recuperar de lá:",
    deleteRecycledChosen:
      "A pasta de cada bot vai para a Lixeira, de onde ainda dá para recuperar. A pasta de trabalho que você escolheu fica onde está:",
    failed: {
      pause: "Não foi possível pausar a equipe",
      resume: "Não foi possível retomar a equipe",
      changeFolder: "Não foi possível mudar a pasta",
      openFolder: "Não foi possível abrir a pasta",
      archive: "Não foi possível arquivar a equipe",
      delete: "Não foi possível excluir a equipe",
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
