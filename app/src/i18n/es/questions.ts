// Preguntas de los bots al dueño (spec 23).

import type { Messages } from "../en";

export const questions: Messages["questions"] = {
  card: {
    asks: (bot: string) => `${bot} pregunta`,
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
  },
  box: {
    label: "Preguntas",
    open: (count: number) =>
      count === 0
        ? "Preguntas de tus bots"
        : count === 1
          ? "1 pregunta te espera"
          : `${count} preguntas te esperan`,
    title: "Preguntas de tus bots",
    intro:
      "Cuando un bot necesita una decisión tuya para seguir, pregunta aquí y sigue trabajando. Tu respuesta le llega como mensaje.",
    emptyTitle: "Ninguna pregunta te espera",
    emptyBody:
      "Los bots preguntan aquí cuando te necesitan, incluso de madrugada, en una rutina. Responde cuando puedas.",
    openChat: (bot: string) => `Abrir el chat con ${bot}`,
    inCrew: (crew: string) => `en ${crew}`,
  },
  activity: (text: string) => `Pregunta: ${text}`,
};
