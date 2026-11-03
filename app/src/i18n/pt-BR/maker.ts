// O Criador (spec 36): o chat, os planos e as configurações dele.

import type { Messages } from "../en";

export const maker: Messages["maker"] = {
  emptyTitle: "Me conte o que você faz, e eu monto as equipes para você",
  emptyBody:
    "Descreva seu trabalho em poucas linhas: o que você faz todo dia, o que se repete, o que gostaria de passar adiante. Eu monto equipes, agentes e rotinas prontos para usar, e você confere tudo antes de qualquer coisa ser criada.",
  startHere: "Deixar o Criador montar",
  startHereHint: "Conte a ele sobre o seu trabalho, e ele monta as equipes para você.",
  card: {
    title: (title) => `O Criador quer montar: ${title}`,
    why: "Por quê",
    newCrew: "Equipe nova",
    include: (name) => `Incluir ${name}`,
    goal: "Meta",
    goalHint: "As instruções do chefe.",
    memory: "Memória da equipe",
    memoryHint: "Todo agente da equipe lê isto.",
    chief: (name) => `${name}, o chefe`,
    agents: "Agentes",
    routines: "Roda sozinho",
    forAgent: (agent) => `Para ${agent}`,
    runs: (count) =>
      count === 1
        ? "1 rotina vai rodar sozinha. Cada vez gasta um pouco do seu plano."
        : `${count} rotinas vão rodar sozinhas. Cada vez gasta um pouco do seu plano.`,
    nothingLeft: "Deixe ao menos uma equipe no plano.",
    build: "Montar",
    decline: "Agora não",
    noteLabel: "O que dizer ao Criador se você recusar",
    notePlaceholder: "Se recusar, diga ao Criador o porquê (opcional)",
    buildFailed: "Não foi possível montar o plano",
    declineFailed: "Não foi possível enviar a resposta",
    built: (title) => `Você montou: ${title}`,
    builtAlone: (title) => `O Criador montou: ${title}`,
    declined: (title) => `Você não montou: ${title}`,
    expired: (title) => `${title} não foi respondido a tempo`,
    open: "Abrir",
    undo: "Desfazer",
    undoTitle: "Desfazer o que o Criador montou?",
    undoBody: (count) =>
      count === 1
        ? "A equipe vai para o arquivo com os agentes e as rotinas dela. Você a traz de volta em Configurações > Arquivados."
        : `As ${count} equipes vão para o arquivo com os agentes e as rotinas delas. Você as traz de volta em Configurações > Arquivados.`,
    undone: "Desfeito: as equipes estão no arquivo",
    undoFailed: "Não foi possível desfazer",
  },
  settings: {
    title: "O Criador",
    show: "Mostrar o Criador",
    showOn: "Ele fica no topo da lista à esquerda.",
    showOff: "Escondido da lista. Nada dele é apagado.",
    withoutAsking: "Deixar o Criador montar sem perguntar",
    withoutAskingOn: "Ele monta os planos na hora; você desfaz cada um por uma semana.",
    withoutAskingOff: "Você confere cada plano antes de qualquer coisa ser criada.",
    confirmTitle: "Deixar o Criador montar sem perguntar?",
    confirmBody:
      "Ele vai criar equipes, agentes e rotinas assim que planejar, sem mostrar antes. Arquivar ou pausar um agente continua perguntando, e você desfaz cada montagem por uma semana.",
    confirm: "Deixar montar",
  },
  history: {
    open: "Trazer meu histórico do Claude Code",
    title: "Trazer seu histórico do Claude Code",
    intro:
      "O Botloft lê o histórico que o Claude Code guarda neste computador e faz um resumo para o Criador. Você escolhe os projetos e lê o resumo inteiro antes de qualquer coisa sair.",
    takes:
      "Só os pedidos que você digitou, os títulos das sessões, as ferramentas usadas, seus CLAUDE.md e os nomes das suas skills. Nunca as respostas do Claude, o que as ferramentas leram ou devolveram, nem anexos.",
    loading: "Lendo seu histórico do Claude Code…",
    loadingHint:
      "Com muitos projetos, isso leva até um minuto. Pode deixar esta janela aberta e esperar.",
    scanError:
      "Não foi possível ler o histórico. Às vezes a primeira leitura demora: tente de novo.",
    retry: "Tentar de novo",
    elapsed: (seconds: number) => `${seconds} s`,
    none: "O Claude Code ainda não tem histórico neste computador.",
    scanFailed: "Não foi possível ler o histórico",
    all: "Todos os projetos",
    project: (sessions: number, prompts: number, period: string) =>
      `${sessions} ${sessions === 1 ? "sessão" : "sessões"} · ${prompts} ${prompts === 1 ? "pedido" : "pedidos"} · ${period}`,
    make: "Montar o resumo",
    making: "Montando o resumo…",
    makingHint:
      "Lendo os projetos escolhidos e trocando chaves, e-mails e números. Isso leva até um minuto.",
    makeFailed: "Não foi possível montar o resumo",
    size: (thousands: number) =>
      `Perto de ${thousands} mil tokens do seu plano quando o Criador ler: como uma conversa longa.`,
    swapped: "Trocados antes de mostrar a você",
    kinds: {
      key: (count: number) => `${count} ${count === 1 ? "chave ou senha" : "chaves ou senhas"}`,
      email: (count: number) => `${count} ${count === 1 ? "e-mail" : "e-mails"}`,
      number: (count: number) => `${count} ${count === 1 ? "número" : "números"}`,
      card: (count: number) => `${count} ${count === 1 ? "cartão" : "cartões"}`,
    },
    notPerfect: "A troca não é perfeita: leia antes de mandar.",
    cut: "Ficaram de fora para caber no limite",
    projects: "Projetos no resumo",
    remake: "Montar de novo sem os desmarcados",
    after:
      "Depois de mandado, o resumo vira um arquivo na conversa do Criador e sai com ela. Um resumo que você não manda é esquecido depois de um dia.",
    forget: "Esquecer",
    send: "Mandar ao Criador",
    sendFailed: "Não foi possível mandar o resumo",
    sent: "Mandado ao Criador",
  },
  claudeAi: {
    open: "Trazer minha exportação do claude.ai",
    title: "Trazer sua exportação do claude.ai",
    intro:
      "O Botloft não entra no claude.ai: você baixa lá uma cópia das suas conversas e escolhe o arquivo aqui.",
    steps: [
      "No claude.ai, abra Configurações > Privacidade e clique em Exportar dados.",
      "Abra o e-mail do claude.ai e baixe o ZIP. O link vale por 24 horas.",
      "Escolha o ZIP aqui. O Botloft lê o arquivo onde ele está, sem copiar.",
    ],
    openSite: "Abrir o claude.ai",
    pick: "Escolher o ZIP",
    other: "Escolher outro arquivo",
    pickTitle: "Exportação do claude.ai",
    kind: "Arquivo ZIP",
    takes:
      "De cada conversa, só o título, a data, quantas mensagens tem e as três primeiras mensagens que você escreveu. Nunca as respostas do Claude nem os anexos.",
    scanFailed: "Não foi possível ler este arquivo",
    reading: "Lendo o arquivo…",
    readingHint: "Uma exportação grande leva até um minuto. O Botloft lê o arquivo onde ele está.",
    found: (count: number, period: string) =>
      `${count} ${count === 1 ? "conversa" : "conversas"} · ${period}`,
    projects: (names: string) => `Projetos: ${names}`,
    periodLabel: "Período",
    periods: {
      all: "Tudo",
      year: "Último ano",
      half: "Últimos 6 meses",
      quarter: "Últimos 3 meses",
    },
    cut: (count: number) =>
      `${count} ${count === 1 ? "conversa mais antiga ficou" : "conversas mais antigas ficaram"} de fora para caber no limite`,
  },
};
