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
      hint: "O Claude Code não está conectado a uma conta, ou a conta não pode ser usada. Entre no Claude e o agente volta a iniciar sozinho. Já entrou? Confira seu plano do Claude e depois reinicie o agente.",
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
    hint: (crew: string) => `Lidera ${crew}: planeja o trabalho e sugere agentes novos`,
  },
  header: {
    noRole: "Sem função definida",
    pause: "Pausar",
    resume: "Retomar",
    restart: "Reiniciar",
    showDetails: "Mostrar detalhes",
    hideDetails: "Ocultar detalhes",
    more: "Mais ações do agente",
    menuOf: (name: string) => `Ações de ${name}`,
    edit: "Editar",
    restartFresh: "Reiniciar com uma nova conversa",
    openFolder: "Abrir pasta",
    makeChief: "Tornar chefe da equipe",
    stopChief: "Deixar de ser chefe",
    markUnread: "Marcar como não lida",
    markRead: "Marcar como lida",
    archive: "Arquivar agente",
    delete: "Excluir agente",
    failed: {
      pause: "Não foi possível pausar o agente",
      resume: "Não foi possível retomar o agente",
      restart: "Não foi possível reiniciar o agente",
      openFolder: "Não foi possível abrir a pasta",
      chief: "Não foi possível trocar o chefe",
      archive: "Não foi possível arquivar o agente",
      delete: "Não foi possível excluir o agente",
    },
    fresh: {
      title: "Começar uma nova conversa?",
      confirm: "Reiniciar",
      body: (name: string) =>
        `${name} reinicia sem a conversa atual. A pasta e o CLAUDE.md dele continuam como estão.`,
    },
    archiveConfirm: {
      title: (name: string) => `Arquivar ${name}?`,
      confirm: "Arquivar agente",
      body: "O agente para e sai da equipe. As mensagens que ainda esperam por ele não são entregues.",
    },
    deleteConfirm: {
      title: (name: string) => `Excluir ${name}?`,
      confirm: "Excluir agente",
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
    newTitle: "Novo agente",
    editTitle: (name: string) => `Editar ${name}`,
    create: "Criar agente",
    save: "Salvar",
    name: "Nome",
    namePlaceholder: "Revisor",
    nameHint: "Os outros agentes falam com ele pelo @ criado a partir deste nome.",
    role: "Função",
    rolePlaceholder: "Revisa pull requests antes do merge",
    instructions: "Instruções",
    instructionsPlaceholder:
      "Como este agente trabalha, o que ele pode fazer sozinho e quando deve perguntar.",
    instructionsHint: "Salvas agora nas regras do agente; ele as lê na próxima vez que iniciar.",
    fullAccess: {
      title: "Rodar qualquer comando sem perguntar",
      off: "Desligado: ele só roda os comandos da lista abaixo.",
      on: "Ligado: ele roda qualquer comando sem perguntar a você, e um comando pode ler ou mudar tudo o que você pode. Só para um agente em que você confia.",
    },
    commands: {
      title: "Comandos que este agente pode rodar",
      placeholder: "git status\nnpm test",
      hint: "Um comando por linha; ele pode rodar estes e o que começar com eles, sem perguntar a você. Ele não consegue perguntar, então todo o resto é recusado. Um * sozinho deixa rodar qualquer coisa. O agente reinicia para usar a lista.",
    },
    agent: {
      title: "Roda em",
      names: {
        claude: "Claude Code",
        agy: "Antigravity (experimental)",
        codex: "Codex",
      },
      experimental:
        "Experimental. Este agente não consegue perguntar a você antes de agir. Ele trabalha com arquivos das próprias pastas e usa as ferramentas da equipe, mas não roda comandos, e não tem medidor de uso nem escolha de modelo.",
      experimentalCodex:
        "Experimental. Este agente pergunta a você antes de mudar um arquivo ou rodar um comando, como o Claude Code. Ele ainda pode ler qualquer arquivo.",
    },
    color: "Cor",
    swatch: (color: string) => `Cor ${color}`,
    colorUnset: "Se você não escolher, o agente recebe a próxima cor da equipe.",
    custom: "Escolher qualquer cor",
    picker: {
      area: "Saturação e brilho",
      areaValue: (saturation: string, brightness: string) =>
        `Saturação ${saturation}, brilho ${brightness}`,
      hue: "Matiz",
      hex: "Hex",
      channels: { r: "R", g: "G", b: "B" },
    },
  },
  notices: {
    crewPaused: (crew: string) => `A equipe ${crew} está pausada`,
    crewPausedBody: "Os agentes dela ficam parados até você retomar a equipe.",
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
    always: "Permitido sem perguntar",
    alwaysNone: (bot: string) =>
      `Nada ainda. Quando ${bot} pedir algo, "Permitir sempre" coloca aqui.`,
    alwaysRemove: (what: string) => `Voltar a perguntar: ${what}`,
    alwaysRemoveFailed: "Não foi possível remover",
    crews: "Outras equipes",
    crewsNone: (bot) =>
      `${bot} só alcança a própria equipe. Quando pedir outra, "Sempre" a coloca aqui.`,
    crewsWhole: (crew) => `A equipe ${crew} inteira`,
    crewsBot: (bot, crew) => `${bot}, da equipe ${crew}`,
    crewsRemove: (what) => `Tirar o acesso: ${what}`,
    crewsRemoveFailed: "Não foi possível tirar o acesso",
    crewsKinds: { talk: "conversar", read: "ler arquivos", edit: "editar arquivos" },
  },
};
