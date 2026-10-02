import type { Messages } from "../en";

export const bots: Messages["bots"] = {
  states: {
    offline: { label: "Desligado", hint: "Não está rodando." },
    launching: { label: "Iniciando", hint: "O Claude Code está iniciando." },
    idle: { label: "Disponível", hint: "Pronto para trabalhar." },
    busy: { label: "Trabalhando", hint: "Trabalhando em algo." },
    needs_approval: {
      label: "Precisa de aprovação",
      hint: "Esperando você permitir ou negar uma ferramenta no chat dele.",
    },
    rate_limited: {
      label: "Limite de uso",
      hint: "Seu plano do Claude atingiu o limite de uso; as mensagens esperam até ele renovar.",
    },
    auth_error: {
      label: "Precisa entrar",
      hint: "O Claude Code não está conectado a uma conta, ou a conta não pode ser usada. Entre no Claude e o bot volta a iniciar sozinho. Já entrou? Confira seu plano do Claude e depois reinicie o bot.",
    },
    backoff: {
      label: "Reiniciando",
      hint: "Parou de forma inesperada; o Botloft inicia de novo em instantes.",
    },
    archived: { label: "Arquivado", hint: "Arquivado." },
    paused: { label: "Pausado", hint: "Pausado; retome para iniciar." },
  },
  chief: {
    badge: "Chefe",
    hint: (crew: string) => `Lidera ${crew}: planeja o trabalho e sugere bots novos`,
  },
  header: {
    noRole: "Sem função definida",
    pause: "Pausar",
    resume: "Retomar",
    restart: "Reiniciar",
    showDetails: "Mostrar detalhes",
    hideDetails: "Ocultar detalhes",
    more: "Mais ações do bot",
    menuOf: (name: string) => `Ações de ${name}`,
    edit: "Editar",
    restartFresh: "Reiniciar com uma nova conversa",
    openFolder: "Abrir pasta",
    makeChief: "Tornar chefe da equipe",
    stopChief: "Deixar de ser chefe",
    markUnread: "Marcar como não lida",
    markRead: "Marcar como lida",
    archive: "Arquivar bot",
    delete: "Excluir bot",
    failed: {
      pause: "Não foi possível pausar o bot",
      resume: "Não foi possível retomar o bot",
      restart: "Não foi possível reiniciar o bot",
      openFolder: "Não foi possível abrir a pasta",
      chief: "Não foi possível trocar o chefe",
      archive: "Não foi possível arquivar o bot",
      delete: "Não foi possível excluir o bot",
    },
    fresh: {
      title: "Começar uma nova conversa?",
      confirm: "Reiniciar",
      body: (name: string) =>
        `${name} reinicia sem a conversa atual. A pasta e o CLAUDE.md dele continuam como estão.`,
    },
    archiveConfirm: {
      title: (name: string) => `Arquivar ${name}?`,
      confirm: "Arquivar bot",
      body: "O bot para e sai da equipe. As mensagens que ainda esperam por ele não são entregues.",
    },
    deleteConfirm: {
      title: (name: string) => `Excluir ${name}?`,
      confirm: "Excluir bot",
      removed: (name: string, running: boolean) =>
        `${name} ${running ? "para agora e sai" : "sai"} do Botloft de vez, com a conversa, as rotinas e as tarefas de que fazia parte. Não dá para desfazer.`,
      chief: (crew: string) => `${crew} fica sem chefe.`,
      kept: (name: string) =>
        `A pasta de ${name} continua no seu computador, com tudo o que está nela:`,
      recycle: "Mandar esta pasta para a Lixeira",
      recycled: (name: string) =>
        `A pasta de ${name} vai para a Lixeira, de onde ainda dá para recuperar:`,
    },
  },
  dialog: {
    newTitle: "Novo bot",
    editTitle: (name: string) => `Editar ${name}`,
    create: "Criar bot",
    save: "Salvar",
    name: "Nome",
    namePlaceholder: "Revisor",
    nameHint: "Os outros bots falam com ele pelo @ criado a partir deste nome.",
    role: "Função",
    rolePlaceholder: "Revisa pull requests antes do merge",
    instructions: "Instruções",
    instructionsPlaceholder:
      "Como este bot trabalha, o que ele pode fazer sozinho e quando deve perguntar.",
    instructionsHint: "Salvas agora nas regras do bot; ele as lê na próxima vez que iniciar.",
    color: "Cor",
    swatch: (color: string) => `Cor ${color}`,
    colorUnset: "Se você não escolher, o bot recebe a próxima cor da equipe.",
  },
  notices: {
    crewPaused: (crew: string) => `A equipe ${crew} está pausada`,
    crewPausedBody: "Os bots dela ficam parados até você retomar a equipe.",
  },
  details: {
    title: (name: string) => `Sobre ${name}`,
    close: "Fechar detalhes",
    role: "Função",
    noRole: "Nenhuma função definida ainda.",
    folder: "Pasta",
    process: "Processo",
    notStarted: "não iniciado",
    generation: (generation: number) => `geração ${generation}`,
    instructions: "Instruções",
    noInstructions: "Nenhuma ainda.",
  },
};
