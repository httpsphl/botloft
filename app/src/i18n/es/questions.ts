// Preguntas de los bots al dueño (spec 23) y sus notas (23.10).

import type { Messages } from "../en";

export const questions: Messages["questions"] = {
  card: {
    asks: (bot: string) => `${bot} pregunta`,
    question: "Pregunta",
    pick: "Elige una respuesta",
    answerLabel: (bot: string) => `Tu respuesta para ${bot}`,
    placeholder: "Escribe tu respuesta",
    send: "Responder",
    dismiss: "Descartar",
    dismissHint: (bot: string) =>
      `Cierra la pregunta sin avisar a ${bot}. Para que lo sepa, responde.`,
    answered: "Respondiste",
    dismissed: "Descartada sin respuesta",
    answerFailed: "No se pudo enviar la respuesta",
    dismissFailed: "No se pudo descartar la pregunta",
    tells: (bot: string) => `${bot} te dejó una nota`,
    note: "Nota",
    replyLabel: (bot: string) => `Tu respuesta para ${bot}`,
    replyPlaceholder: "Responde, si quieres",
    reply: "Responder",
    seen: "Visto",
    seenHint: (bot: string) => `Cierra la nota. Escribir a ${bot} en su chat también la cierra.`,
    replied: "Respondiste",
    wasSeen: "Vista",
    seenFailed: "No se pudo cerrar la nota",
  },
  box: {
    label: "Bandeja de entrada",
    open: (count: number) =>
      count === 0
        ? "Preguntas y notas de tus agentes"
        : count === 1
          ? "1 pregunta o nota te espera"
          : `${count} preguntas y notas te esperan`,
    title: "Bandeja de entrada",
    intro:
      "Cuando un agente necesita una decisión tuya, o tiene algo que debes saber, lo deja aquí y sigue trabajando. Tu respuesta le llega como mensaje.",
    emptyTitle: "Nada te espera",
    emptyBody:
      "Los agentes preguntan y avisan aquí, incluso de madrugada, en una rutina. Responde cuando puedas.",
    openChat: (bot: string) => `Abrir el chat con ${bot}`,
    inCrew: (crew: string) => `en ${crew}`,
  },
  activity: (text: string) => `Pregunta: ${text}`,
  noteActivity: (text: string) => `Nota: ${text}`,
};
