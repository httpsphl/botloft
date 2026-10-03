// Perguntas dos bots ao dono (spec 23).

import type { Messages } from "../en";

export const questions: Messages["questions"] = {
  card: {
    asks: (bot: string) => `${bot} pergunta`,
    pick: "Escolha uma resposta",
    answerLabel: (bot: string) => `Sua resposta para ${bot}`,
    placeholder: "Escreva sua resposta",
    send: "Responder",
    dismiss: "Descartar",
    dismissHint: (bot: string) => `Fecha a pergunta sem avisar ${bot}. Para ele saber, responda.`,
    answered: "Você respondeu",
    dismissed: "Descartada sem resposta",
    answerFailed: "Não foi possível enviar a resposta",
    dismissFailed: "Não foi possível descartar a pergunta",
  },
  box: {
    label: "Perguntas",
    open: (count: number) =>
      count === 0
        ? "Perguntas dos seus bots"
        : count === 1
          ? "1 pergunta espera você"
          : `${count} perguntas esperam você`,
    title: "Perguntas dos seus bots",
    intro:
      "Quando um bot precisa de uma decisão sua para continuar, ele pergunta aqui e segue o trabalho. Sua resposta chega a ele como mensagem.",
    emptyTitle: "Nenhuma pergunta esperando você",
    emptyBody:
      "Os bots perguntam aqui quando precisam de você, até de madrugada, numa rotina. Responda quando puder.",
    openChat: (bot: string) => `Abrir a conversa com ${bot}`,
    inCrew: (crew: string) => `em ${crew}`,
  },
  activity: (text: string) => `Pergunta: ${text}`,
};
