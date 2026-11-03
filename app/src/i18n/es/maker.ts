// El Creador (spec 36): su chat, sus planes y sus ajustes.

import type { Messages } from "../en";

export const maker: Messages["maker"] = {
  emptyTitle: "Cuéntame qué haces, y armo tus equipos",
  emptyBody:
    "Describe tu trabajo en pocas líneas: qué haces cada día, qué se repite, qué te gustaría delegar. Armo equipos, agentes y rutinas listos para usar, y tú revisas todo antes de que se cree nada.",
  startHere: "Dejar que el Creador arme",
  startHereHint: "Cuéntale de tu trabajo y arma los equipos por ti.",
  card: {
    title: (title) => `El Creador quiere armar: ${title}`,
    why: "Por qué",
    newCrew: "Equipo nuevo",
    include: (name) => `Incluir ${name}`,
    goal: "Meta",
    goalHint: "Las instrucciones del jefe.",
    memory: "Memoria del equipo",
    memoryHint: "Todos los agentes del equipo leen esto.",
    chief: (name) => `${name}, el jefe`,
    agents: "Agentes",
    routines: "Corre solo",
    forAgent: (agent) => `Para ${agent}`,
    runs: (count) =>
      count === 1
        ? "1 rutina correrá sola. Cada vez usa un poco de tu plan."
        : `${count} rutinas correrán solas. Cada vez usa un poco de tu plan.`,
    nothingLeft: "Deja al menos un equipo en el plan.",
    build: "Armar",
    decline: "Ahora no",
    noteLabel: "Qué decirle al Creador si dices que no",
    notePlaceholder: "Si dices que no, dile al Creador por qué (opcional)",
    buildFailed: "No se pudo armar el plan",
    declineFailed: "No se pudo enviar la respuesta",
    built: (title) => `Armaste: ${title}`,
    builtAlone: (title) => `El Creador armó: ${title}`,
    declined: (title) => `No armaste: ${title}`,
    expired: (title) => `${title} no se respondió a tiempo`,
    open: "Abrir",
    undo: "Deshacer",
    undoTitle: "¿Deshacer lo que armó el Creador?",
    undoBody: (count) =>
      count === 1
        ? "El equipo va al archivo con sus agentes y rutinas. Lo traes de vuelta en Ajustes > Archivados."
        : `Los ${count} equipos van al archivo con sus agentes y rutinas. Los traes de vuelta en Ajustes > Archivados.`,
    undone: "Deshecho: los equipos están en el archivo",
    undoFailed: "No se pudo deshacer",
  },
  settings: {
    title: "El Creador",
    show: "Mostrar el Creador",
    showOn: "Queda arriba de la lista a la izquierda.",
    showOff: "Oculto de la lista. No se borra nada de él.",
    withoutAsking: "Dejar que el Creador arme sin preguntar",
    withoutAskingOn: "Arma sus planes enseguida; puedes deshacer cada uno durante una semana.",
    withoutAskingOff: "Revisas cada plan antes de que se cree nada.",
    confirmTitle: "¿Dejar que el Creador arme sin preguntar?",
    confirmBody:
      "Creará equipos, agentes y rutinas en cuanto los planee, sin mostrártelos antes. Archivar o pausar un agente sigue preguntando, y puedes deshacer cada armado durante una semana.",
    confirm: "Dejar que arme",
  },
};
