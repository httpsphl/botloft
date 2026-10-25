// A ilha no topo da tela (spec 31).

import type { Messages } from "../en";

export const island: Messages["island"] = {
  label: "Botloft no topo da tela",
  waiting: (count) => (count === 1 ? "1 esperando" : `${count} esperando`),
  working: (count) => (count === 1 ? "1 trabalhando" : `${count} trabalhando`),
  position: (at, of) => `${at} de ${of}`,
  previous: "Anterior",
  next: "Próximo",
  tabs: {
    label: "Abas da ilha",
    home: "Início",
  },
  openApp: "Abrir o Botloft",
  hide: "Esconder",
  card: {
    allow: "Permitir",
    deny: "Negar",
    openInApp: "Abrir no Botloft",
    cut: "Longo demais para aparecer inteiro aqui. Abra no Botloft para ler tudo.",
    atComputer: "Precisa de você no Botloft",
    asks: (bot) => `${bot} pergunta`,
    answerLabel: (bot) => `Sua resposta para ${bot}`,
    placeholder: "Escreva sua resposta",
    send: "Responder",
    failed: "Não deu para enviar. Tente de novo, ou abra no Botloft.",
  },
};
