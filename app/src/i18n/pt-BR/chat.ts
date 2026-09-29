import type { Messages } from "../en";

export const chat: Messages["chat"] = {
  view: {
    label: (bot: string) => `Chat com ${bot}`,
    loadFailed: "Não foi possível carregar o chat",
    loadEarlier: "Carregar mensagens anteriores",
    loading: "Carregando o chat…",
    emptyTitle: (bot: string) => `Comece uma conversa com ${bot}`,
    emptyBody:
      "Peça qualquer coisa que a pasta e as ferramentas dele permitam fazer. Você também pode anexar arquivos e imagens.",
    messages: "Mensagens",
    dropToAttach: "Solte para anexar à sua mensagem",
  },
  composer: {
    paused: (bot: string) =>
      `${bot} está pausado. O que você enviar fica esperando até ele voltar a rodar.`,
    filesToSend: "Arquivos para enviar",
    remove: (file: string) => `Remover ${file}`,
    label: (bot: string) => `Mensagem para ${bot}`,
    placeholder: (bot: string) => `Escreva para ${bot}`,
    attach: "Anexar arquivos",
    attachHint: "Anexar arquivos (ou cole, ou solte no chat)",
    tooLong: (length: number, max: number) => `${length}/${max} caracteres`,
    keys: "Enter para enviar, Shift+Enter para uma nova linha",
    send: "Enviar",
  },
  files: {
    tooMany: (max: number) => `Uma mensagem pode levar até ${max} arquivos.`,
    tooBig: (size: string) =>
      `Os arquivos de uma mensagem podem somar até ${size}. Envie em partes.`,
    unreadable: (file: string) => `não foi possível ler ${file}`,
  },
  attachments: {
    label: "Anexos",
    file: "Arquivo",
    showInFolder: (file: string) => `Mostrar ${file} na pasta`,
    showInFolderHint: "Mostrar na pasta",
    openFolderFailed: "Não foi possível abrir a pasta",
    missing: "Não está mais na pasta do bot",
    loading: (file: string) => `Carregando ${file}`,
  },
  inbound: {
    task: "Tarefa",
    result: "Resultado",
    archivedBot: "Um bot arquivado",
  },
  run: {
    working: "Trabalhando",
    done: (time: string) => `Concluído em ${time}`,
    took: (time: string, usd: number | null) =>
      `Levou ${time}${
        usd === null
          ? ""
          : ` · cerca de ${usd.toLocaleString("pt-BR", { style: "currency", currency: "USD" })} de uso`
      }`,
    stopped: (reason: string) => `O bot parou de trabalhar nisto: ${reason}`,
  },
  tools: {
    label: "Chamadas de ferramentas",
    running: "Rodando",
    done: "Concluído",
    failed: "Falhou",
    input: "Entrada",
    output: "Saída",
    error: "Erro",
  },
  approval: {
    asks: (bot: string, tool: string) => `${bot} pede para usar ${tool}`,
    wants: (bot: string, tool: string) => `${bot} quer usar ${tool}`,
    fullInput: "Entrada completa",
    noteLabel: (bot: string) => `Observação para ${bot} se você negar`,
    notePlaceholder: "Por que não? Vai para o bot se você negar (opcional)",
    allow: "Permitir",
    deny: "Negar",
    allowFailed: "Não foi possível permitir",
    denyFailed: "Não foi possível negar",
    allowed: (tool: string) => `Você permitiu ${tool}`,
    denied: (tool: string) => `Você negou ${tool}`,
    expired: (tool: string) => `${tool} não foi aprovado a tempo`,
  },
  mode: {
    title: "Modo",
    button: (mode: string) => `Modo: ${mode}`,
    names: {
      auto: "Automático",
      default: "Manual",
      accept_edits: "Aceitar edições",
      plan: "Plano",
      bypass_permissions: "Ignorar permissões",
    },
    hints: {
      auto: (bot: string) => `${bot} decide o que precisa do seu OK`,
      default: (bot: string) => `${bot} sempre pergunta antes de fazer alterações`,
      accept_edits: (bot: string) => `${bot} edita arquivos sem perguntar`,
      plan: (bot: string) => `${bot} cria um plano antes de fazer alterações`,
      bypass_permissions: (bot: string) => `${bot} faz tudo sem perguntar`,
    },
    turnOn: "Ativar",
    failed: "Não foi possível mudar o modo",
    later: (bot: string, mode: string) =>
      `${bot} muda para ${mode} quando terminar o que está fazendo.`,
    bypassTitle: (bot: string) => `Deixar ${bot} fazer tudo sem perguntar?`,
    bypassBody: (bot: string) =>
      `${bot} vai editar arquivos, rodar comandos e usar a internet neste computador sem pedir sua permissão antes.`,
    bypassRisk:
      "Ele não fica preso à pasta dele: pode ler e alterar seus outros arquivos, os arquivos de outros bots e os do próprio Botloft. Uma mensagem de outra pessoa pode levá-lo a fazer algo que você não queria.",
    bypassAdvice: "Ative só para um bot em quem você confia para tudo, e só enquanto precisar.",
    badge: "Não pergunta nada",
    badgeHint: "Este bot faz tudo sem perguntar. Mude isso no seletor de modo, embaixo do chat.",
  },
  model: {
    title: "Modelo",
    button: (model: string) => `Modelo: ${model}`,
    names: {
      default: "Padrão do plano",
      fable: "Fable",
      opus: "Opus",
      sonnet: "Sonnet",
      haiku: "Haiku",
    },
    short: "Padrão",
    hints: {
      default: (bot: string, inUse: string | null) =>
        inUse
          ? `${bot} usa o modelo padrão do seu plano, hoje o ${inUse}`
          : `${bot} usa o modelo padrão do seu plano`,
      fable: (bot: string) => `${bot} fica no máximo, para o trabalho mais difícil`,
      opus: (bot: string) => `${bot} vai bem em tarefas longas e complexas`,
      sonnet: (bot: string) => `${bot} fica rápido e capaz, bom para quase tudo`,
      haiku: (bot: string) =>
        `${bot} fica mais rápido e gasta menos do seu plano, para tarefas simples`,
    },
    cost: "Modelos mais capazes gastam o limite do seu plano mais rápido.",
    failed: "Não foi possível trocar o modelo",
    later: (bot: string, model: string) =>
      `${bot} muda para ${model} quando terminar o que está fazendo.`,
  },
  plan: {
    ready: (bot: string) => `${bot} fez um plano e quer seguir com ele`,
    noteLabel: (bot: string) => `O que ${bot} deve mudar no plano`,
    notePlaceholder: "O que deve mudar? Vai para o bot se você pedir mudanças (opcional)",
    approve: "Aprovar plano",
    keepPlanning: "Pedir mudanças",
    approveFailed: "Não foi possível aprovar o plano",
    keepFailed: "Não foi possível devolver o plano",
    approved: "Você aprovou o plano",
    sentBack: "Você pediu mudanças no plano",
    expired: "O plano não foi aprovado a tempo",
  },
  markdown: {
    image: "imagem",
    openLinkFailed: "Não foi possível abrir o link",
  },
  notice: {
    signedOut:
      "O Claude Code não está conectado a uma conta, ou a conta não pode ser usada agora. Entre no Claude ou confira o seu plano do Claude.",
    usageLimit:
      "Seu plano do Claude atingiu o limite de uso. As mensagens ficam esperando até o limite renovar.",
    modelUnavailable:
      "O modelo deste bot não está disponível: talvez não faça parte do seu plano do Claude. Escolha outro modelo abaixo do chat.",
    turnFailed: (detail: string) => `O bot não conseguiu terminar isto: ${detail}`,
  },
};
