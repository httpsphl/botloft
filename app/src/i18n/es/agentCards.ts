// Tarjetas que un agente muestra en su chat (spec 8.11).

import type { agentCards as en } from "../en/agentCards";

export const agentCards: typeof en = {
  label: (bot, title) => `Tarjeta de ${bot}: ${title}`,
  updated: (time) => `Actualizada ${time}`,
  send: "Enviar",
  required: "obligatorio",
  pick: "Elige una",
  pressed: (label) => `Elegiste “${label}”`,
  sent: "Enviaste el formulario",
  waiting: (bot) =>
    `${bot} ya tiene tu respuesta. Para preguntar de nuevo, vuelve a mostrar la tarjeta.`,
  failed: "No se pudo enviar. Inténtalo de nuevo.",
  chart: {
    label: (kind, series) => `Gráfico de ${kind} de ${series}`,
    bar: "barras",
    line: "líneas",
    asTable: "Ver como tabla",
    asChart: "Ver como gráfico",
    label0: "Etiqueta",
  },
  activity: (title) => `Tarjeta: ${title}`,
};
