// Perguntas dos bots ao dono (spec 23) e recados deles (23.10).

import type { Messages } from "../en";

export const questions: Messages["questions"] = {
  card: {
    asks: (bot: string) => `${bot} pergunta`,
    question: "Pergunta",
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
    tells: (bot: string) => `${bot} deixou um recado`,
    note: "Recado",
    replyLabel: (bot: string) => `Sua resposta para ${bot}`,
    replyPlaceholder: "Responda, se quiser",
    reply: "Responder",
    seen: "Visto",
    seenHint: (bot: string) => `Fecha o recado. Escrever para ${bot} na conversa também fecha.`,
    replied: "Você respondeu",
    wasSeen: "Visto",
    seenFailed: "Não foi possível fechar o recado",
  },
  box: {
    label: "Caixa de entrada",
    open: (count: number) =>
      count === 0
        ? "Perguntas e recados dos seus agentes"
        : count === 1
          ? "1 pergunta ou recado espera você"
          : `${count} perguntas e recados esperam você`,
    title: "Caixa de entrada",
    intro:
      "Quando um agente precisa de uma decisão sua, ou tem algo que você deve saber, ele deixa aqui e segue o trabalho. Sua resposta chega a ele como mensagem.",
    emptyTitle: "Nada esperando você",
    emptyBody:
      "Os agentes perguntam e avisam aqui, até de madrugada, numa rotina. Responda quando puder.",
    openChat: (bot: string) => `Abrir a conversa com ${bot}`,
    inCrew: (crew: string) => `em ${crew}`,
  },
  activity: (text: string) => `Pergunta: ${text}`,
  noteActivity: (text: string) => `Recado: ${text}`,
};
