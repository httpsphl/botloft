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
};
