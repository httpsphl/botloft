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
  markdown: {
    image: "imagem",
    openLinkFailed: "Não foi possível abrir o link",
  },
  notice: {
    signedOut:
      "O Claude Code não está conectado a uma conta, ou a conta não pode ser usada agora. Entre no Claude ou confira o seu plano do Claude.",
    usageLimit:
      "Seu plano do Claude atingiu o limite de uso. As mensagens ficam esperando até o limite renovar.",
    turnFailed: (detail: string) => `O bot não conseguiu terminar isto: ${detail}`,
  },
};
