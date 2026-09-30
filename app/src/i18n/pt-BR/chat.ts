import type { Messages } from "../en";
import type { TurnTokens } from "../en/chat";

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
    keysWithCtrl: "Ctrl+Enter para enviar, Enter para uma nova linha",
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
    goneBot: "Um bot que não está mais na equipe",
  },
  run: {
    working: "Trabalhando",
    done: (time: string, tokens: string | null) =>
      `Concluído em ${time}${tokens === null ? "" : ` · ${tokens} tokens`}`,
    took: (time: string, tokens: TurnTokens | null) =>
      `Levou ${time}${
        tokens === null
          ? ""
          : `. Leu ${tokens.read} tokens novos e escreveu ${tokens.wrote}; releu ${tokens.reread} da conversa, que pesa bem menos.`
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
    command: "Comando",
    commandCut: "Este comando é longo demais para aparecer inteiro.",
  },
  approval: {
    asks: (bot: string, action: string) => `${bot} pede para ${action}`,
    wants: (bot: string, action: string) => `${bot} quer ${action}`,
    explainedBy: (bot: string) => `${bot} escreveu isto. O que roda de verdade é o comando abaixo.`,
    unexplained: (bot: string) =>
      `${bot} não disse para que serve este comando. Na dúvida, negue e pergunte.`,
    command: "Ver o comando",
    noteLabel: (bot: string) => `Observação para ${bot} se você negar`,
    notePlaceholder: "Por que não? Vai para o bot se você negar (opcional)",
    allow: "Permitir",
    deny: "Negar",
    allowFailed: "Não foi possível permitir",
    denyFailed: "Não foi possível negar",
    allowed: (action: string) => `Permitido: ${action}`,
    denied: (action: string) => `Negado: ${action}`,
    expired: (action: string) => `Sem resposta a tempo: ${action}`,
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
    bypassChief: (bot: string) =>
      `${bot} é o chefe da equipe, então também vai criar bots novos sem perguntar.`,
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
  effort: {
    title: "Esforço",
    button: (level: string) => `Esforço: ${level}`,
    short: "Esforço",
    faster: "Mais rápido",
    smarter: "Mais inteligente",
    names: {
      low: "Baixo",
      medium: "Médio",
      high: "Alto",
      xhigh: "Extra alto",
      max: "Máximo",
    },
    hints: {
      low: (bot: string) => `${bot} responde mais rápido e pensa menos, para tarefas simples`,
      medium: (bot: string) => `${bot} equilibra rapidez e raciocínio, bom para quase tudo`,
      high: (bot: string) => `${bot} pensa mais antes de responder, para tarefas mais difíceis`,
      xhigh: (bot: string) => `${bot} pensa bem mais, para trabalho longo e complexo`,
      max: (bot: string) =>
        `${bot} pensa o máximo que pode: é o mais lento e o que mais gasta do seu plano`,
    },
    recommended: "Recomendado",
    recommendedFor: (model: string) => `Recomendado para o ${model}`,
    useRecommended: "Usar o recomendado",
    unknown: (bot: string) => `${bot} usa o nível recomendado para o modelo dele`,
    unavailable: "Indisponível",
    none: (bot: string, model: string) =>
      `${model} não tem níveis de esforço: ${bot} responde sempre no mesmo ritmo.`,
    thisModel: "Este modelo",
    cost: "Mais esforço gasta o limite do seu plano mais rápido.",
    failed: "Não foi possível mudar o esforço",
    later: (bot: string, level: string) =>
      `${bot} muda para o esforço ${level} quando terminar o que está fazendo.`,
  },
  context: {
    title: "Espaço da conversa",
    button: (used: string, total: string, percent: number) =>
      `Espaço da conversa: ${used} de ${total} em uso (${percent}%)`,
    used: (used: string, total: string, percent: number) => `${used} / ${total} (${percent}%)`,
    about: (bot: string) =>
      `Tudo o que ${bot} leu e escreveu nesta conversa ocupa espaço. Quanto mais cheia, mais do seu plano cada mensagem gasta.`,
    autoLeft: (left: string, at: string) =>
      `Faltam ${left} para ela se compactar sozinha, em ${at}.`,
    autoNow: "Já está cheia o bastante para se compactar sozinha na próxima mensagem.",
    noAuto: "Ela não se compacta sozinha.",
    compact: "Compactar agora",
    compactHint: (bot: string) =>
      `Troca o que veio antes por um resumo: ${bot} volta a ter espaço e cada mensagem gasta menos.`,
    compacting: "Compactando…",
    later: (bot: string) => `${bot} compacta a conversa quando terminar o que está fazendo.`,
    notRunning: (bot: string) => `${bot} não está rodando, então não dá para compactar agora.`,
    failed: "Não foi possível compactar a conversa",
  },
  suggestion: {
    title: (bot: string) => `${bot} sugere um bot novo`,
    why: "Por quê",
    name: "Nome",
    role: "Função",
    model: "Modelo",
    instructions: "Instruções",
    startsNow: (bot: string) => `Ele começa na hora e recebe o trabalho de ${bot}.`,
    noteLabel: (bot: string) => `O que dizer a ${bot} se você recusar`,
    notePlaceholder: (bot: string) => `Se recusar, diga a ${bot} o porquê (opcional)`,
    create: "Criar bot",
    decline: "Agora não",
    createFailed: "Não foi possível criar o bot",
    declineFailed: "Não foi possível enviar a resposta",
    created: (name: string) => `Você criou ${name}`,
    declined: (name: string) => `Você recusou ${name}`,
    expired: (name: string) => `${name} não foi respondido a tempo`,
  },
  routineRequest: {
    title: (bot: string) => `${bot} quer criar uma rotina`,
    titleFor: (bot: string, runner: string) => `${bot} quer criar uma rotina para ${runner}`,
    explain: (runner: string) =>
      `Uma rotina faz ${runner} trabalhar sozinho em horários marcados. Ela aparece na aba Rotinas, onde você pode pausar ou apagar.`,
    noteLabel: (bot: string) => `O que dizer a ${bot} se você recusar`,
    notePlaceholder: (bot: string) => `Se recusar, diga a ${bot} o porquê (opcional)`,
    create: "Criar rotina",
    decline: "Agora não",
    createFailed: "Não foi possível criar a rotina",
    declineFailed: "Não foi possível enviar a resposta",
    created: (name: string) => `Você criou a rotina ${name}`,
    declined: (name: string) => `Você recusou a rotina ${name}`,
    expired: (name: string) => `A rotina ${name} não foi respondida a tempo`,
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
    compacted:
      "A conversa foi compactada: o que veio antes agora é um resumo, e há espaço de novo.",
    autoCompacted: "A conversa ficou cheia e foi compactada: o que veio antes agora é um resumo.",
    compactFailed: (detail: string) => `Não foi possível compactar a conversa: ${detail}`,
  },
};
