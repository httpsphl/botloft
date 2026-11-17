// O que os agentes dizem e mostram nas demonstrações do site (spec 39.4).

import type { SiteMessages } from "../en/site";

export const siteDemos: SiteMessages["demos"] = {
  window: {
    crew: "Pesquisa",
    agents: "4 agentes",
    message: "Mensagem para o Scout",
  },
  states: {
    idle: "Parado",
    working: "Trabalhando",
    approval: "Precisa de aprovação",
    sleeping: "Dormindo",
  },
  roles: {
    scout: "Acha fontes e resume",
    writer: "Transforma notas em posts",
    analyst: "Faz as contas",
    designer: "Desenha as páginas da loja",
  },
  hero: {
    ask: "Ache três boas fontes sobre fermento natural e passe para o Writer escrever o post de sexta.",
    wants: "Scout quer usar o navegador em",
    site: "padeiros.example",
    allow: "Permitir",
    once: "Só desta vez",
    deny: "Negar",
    allowed: "Você permitiu padeiros.example",
    steps: "Fez 4 passos no navegador",
    reply:
      "Achei três: um guia de padeiro para alimentar o fermento, um texto de ciência dos alimentos sobre levedura selvagem e as anotações de um padeiro caseiro. Passando para o Writer.",
    sent: "Mandou uma tarefa para o Writer",
    task: "Escrever o post de sexta com estas três fontes",
  },
  browser: {
    live: "Ao vivo",
    tab: "Receitas",
    page: "Ache uma receita",
    query: "fermento natural",
    search: "Buscar",
    results: [
      "Alimentando o fermento, dia a dia",
      "Por que o fermento fica azedo",
      "Centeio ou trigo para começar?",
    ],
    control: "O Scout está no controle",
    take: "Assumir o controle",
  },
  ask: {
    wants: "Writer quer rodar um comando",
    command: "npm run build",
    allow: "Permitir",
    deny: "Negar",
    allowed: "Você permitiu",
    after: "O build passou. Publicando o rascunho para você revisar.",
  },
  handoff: {
    task: "Escrever o post de sexta",
  },
  routines: {
    items: [
      { name: "Resumo da manhã", when: "Dias úteis, 8:00" },
      { name: "Checagem de links", when: "A cada 2 horas" },
      { name: "Números da semana", when: "Segundas, 9:00" },
    ],
    running: "Rodando",
    done: "Feita, 8:00",
  },
  phone: {
    time: "9:41",
    notice: "Writer precisa de você",
    noticeBody: "Quer rodar npm run build",
    allow: "Permitir",
    deny: "Negar",
    allowed: "Permitido",
  },
  maker: {
    ask: "Monte uma equipe para a loja online da minha padaria.",
    plan: "O plano",
    items: [
      "Equipe: Loja da padaria, com um Chefe",
      "Designer: desenha as páginas da loja",
      "Atendente: recebe os pedidos",
      "Rotina: resumo dos pedidos, dias úteis 9:00",
    ],
    build: "Criar",
    change: "Mudar",
    built: "Criado. Desfazer volta atrás.",
  },
  sleep: {
    message: "Mensagem nova",
  },
};
