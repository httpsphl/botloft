// La vista general junto al chat del agente (spec 33).

import type { Messages } from "../en";

export const overview: Messages["overview"] = {
  heading: "Vista general",
  panel: (bot) => `Vista general de ${bot}`,
  show: "Mostrar vista general",
  hide: "Ocultar vista general",
  close: "Cerrar",
  context: {
    title: "Espacio de la conversación",
    used: (percent) => `${percent} ocupado`,
    of: (used, total) => `${used} de ${total}`,
    bar: (percent) => `Espacio de la conversación, ${percent} ocupado`,
    unknown: (bot) => `${bot} todavía no dijo cuánto de la conversación está ocupado.`,
    noParts: "Este motor solo da el total, no lo que lo ocupa.",
    parts: "Lo que lo ocupa",
    free: "Libre",
    kind: {
      messages: "Conversación",
      instructions: "Instrucciones de Claude Code",
      tools: "Herramientas de Claude Code",
      connected_tools: "Herramientas conectadas",
      memory: "Memoria",
      skills: "Habilidades",
      agents: "Subagentes",
      buffer: "Reserva para compactar",
      other: "Otros",
    },
    memory: "Archivos de memoria",
    memoryHint: (bot) => `Lo que ${bot} lee al comienzo de cada conversación.`,
  },
  background: {
    title: "En segundo plano",
    none: "Nada en marcha en segundo plano.",
    kind: { command: "Comando", agent: "Subagente", remote: "Agente remoto", other: "Tarea" },
    since: (time) => `Desde ${time}`,
    stop: "Detener",
    stopOne: (what) => `Detener: ${what}`,
    stopFailed: "No se pudo detener la tarea",
    hint: (bot) => `${bot} dejó esto en marcha. Termina cuando ${bot} se detiene o se reinicia.`,
  },
};
