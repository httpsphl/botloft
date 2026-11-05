import type { chrome as en } from "../en/chrome";

export const chrome: typeof en = {
  tab: "Tu Chrome",
  short: "Chrome",
  intro:
    "Los agentes que elijas pueden usar tu propio Chrome, con tus cuentas abiertas, mediante la extensión Claude in Chrome. Solo agentes de Claude Code.",
  none: "Ningún agente tiene tu Chrome. Un agente lo pide en su chat cuando una tarea lo necesita.",
  givenOn: (when) => `Dado ${when}`,
  give: "Dar tu Chrome a un agente",
  take: (bot) => `Quitarle tu Chrome a ${bot}`,
  takeFailed: "No se pudo quitar tu Chrome.",
  giveFailed: "No se pudo dar tu Chrome.",
  sites: 'Los sitios que permitiste siempre están en Comandos, como "En tu Chrome".',
  using: (bot) => `${bot} está usando tu Chrome ahora.`,
  nobody: "Ningún agente está usando tu Chrome ahora.",
  stop: "Detener",
  stopFailed: "No se pudo detener al agente.",
  oneAtATime:
    "Un agente a la vez usa tu Chrome, cada uno en sus propias pestañas; tú sigues usándolo como siempre. Detener rechaza los próximos pasos del agente allí hasta que termine su turno.",
  dialog: {
    title: "Dar tu Chrome a un agente",
    agent: "Agente",
    risks: (bot) => [
      `Tu Chrome tiene tus cuentas abiertas: allí, ${bot} actúa como tú.`,
      `${bot} puede equivocarse y enviar, publicar, comprar o borrar algo como si fueras tú.`,
      "Un texto en un sitio puede intentar engañar al agente para que haga otra cosa.",
      "Cada sitio nuevo te pregunta antes, y ves cada paso en el chat.",
    ],
    understood: "Entiendo los riesgos",
    confirm: "Dar mi Chrome",
    cancel: "Cancelar",
    noAgent: "Todos los agentes de Claude Code ya tienen tu Chrome.",
  },
  request: {
    asks: (bot) => `${bot} pide tu Chrome`,
    wants: (bot) => `${bot} quiere usar tu Chrome`,
    give: "Dar mi Chrome",
    given: (bot) => `Le diste tu Chrome a ${bot}`,
    declined: "No diste tu Chrome",
    expired: "La solicitud de tu Chrome venció",
  },
  details: {
    label: "Tu Chrome",
    off: (bot) => `${bot} no usa tu Chrome. Lo pide en su chat cuando una tarea lo necesita.`,
    notClaude: "Solo los agentes de Claude Code pueden usar tu Chrome.",
    give: "Dar mi Chrome",
  },
};
