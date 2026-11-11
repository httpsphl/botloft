// El historial de la memoria y las instrucciones de un agente (spec 8.10).

import type { versions as en } from "../en/versions";

export const versions: typeof en = {
  memoryHistory: "Historial de la memoria",
  instructionsHistory: "Historial de las instrucciones",
  memory: "Memoria",
  memoryAbout: (bot) =>
    `Lo que ${bot} guarda de una conversación a otra, en el CLAUDE.md de su carpeta. ${bot} lo escribe; cada cambio queda guardado.`,
  title: {
    memory: (bot) => `Historial de la memoria de ${bot}`,
    instructions: (bot) => `Historial de las instrucciones de ${bot}`,
  },
  intro: {
    memory: (bot) =>
      `Cada cambio en la memoria de ${bot}, del más nuevo al más antiguo. Botloft guarda este historial fuera de la carpeta de ${bot}, así que ${bot} no puede cambiarlo.`,
    instructions: (bot) =>
      `Cada cambio en las instrucciones de ${bot}, del más nuevo al más antiguo.`,
  },
  empty: {
    memory: (bot) =>
      `Aún no hay cambios. El historial empieza la primera vez que ${bot} u otra persona cambie la memoria.`,
    instructions:
      "Aún no hay cambios. El historial empieza la primera vez que cambien las instrucciones.",
  },
  list: "Versiones",
  author: {
    start: "Como estaba al principio",
    owner: "Lo cambiaste tú",
    restored: (at) => `Volviste a la versión de ${at}`,
    agent: (bot) => `${bot} lo cambió`,
    maker: "El Creador lo cambió",
    outside: "Cambió fuera de Botloft",
  },
  now: "Ahora",
  emptyText: "(vacía)",
  showChanges: "Ver los cambios",
  restore: "Volver a esta versión",
  restoreConfirm: {
    title: "¿Volver a esta versión?",
    memory: (bot, at) =>
      `La memoria de ${bot} vuelve a como estaba el ${at}. La versión de ahora queda en el historial, así que también puedes deshacer esto. ${bot} la lee la próxima vez que empiece una conversación.`,
    instructions: (bot, at) =>
      `Las instrucciones de ${bot} vuelven a como estaban el ${at}. La versión de ahora queda en el historial, así que también puedes deshacer esto.`,
    confirm: "Volver",
  },
  failed: {
    load: "No se pudo cargar el historial",
    restore: "No se pudo volver a esa versión",
  },
};
