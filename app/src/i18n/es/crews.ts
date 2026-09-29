import type { Messages } from "../en";

export const crews: Messages["crews"] = {
  newCrew: "Nuevo equipo",
  newBot: "Nuevo bot",
  rename: "Renombrar",
  paused: "En pausa",
  sidebar: {
    label: "Equipos",
    noMessages: "Aún no hay mensajes",
    fromOwner: (text: string) => `Tú: ${text}`,
    awaitingApproval: (text: string) => `Esperando aprobación: ${text}`,
  },
  dialog: {
    renameTitle: "Renombrar equipo",
    create: "Crear equipo",
    name: "Nombre",
    namePlaceholder: "Investigación",
    createHint: "Los bots de un equipo pueden enviarse mensajes y comparten una carpeta.",
    renameHint: (slug: string) => `La carpeta conserva su nombre (${slug}).`,
  },
  view: {
    bots: (count: number) => (count === 1 ? "1 bot" : `${count} bots`),
    pausedNote: "en pausa: sus bots siguen detenidos hasta que lo reanudes",
    pause: "Pausar equipo",
    resume: "Reanudar equipo",
    moreActions: "Más acciones del equipo",
    openShared: "Abrir carpeta compartida",
    archive: "Archivar equipo",
    tabs: {
      label: "Vistas del equipo",
      bots: "Bots",
      timeline: "Línea de tiempo",
      tasks: "Tareas",
    },
    timelineEmpty:
      "Aún no hay mensajes. Escríbele a un bot abajo; lo que los bots se envían entre sí también aparece aquí.",
    archiveTitle: (crew: string) => `¿Archivar ${crew}?`,
    archiveBody: (bots: number) =>
      bots === 0
        ? "El equipo sale de la app."
        : `El equipo sale de la app. ${bots === 1 ? "Su bot se detiene" : `Sus ${bots} bots se detienen`}, y los mensajes que aún ${bots === 1 ? "lo" : "los"} esperan no se entregan.`,
    failed: {
      pause: "No se pudo pausar el equipo",
      resume: "No se pudo reanudar el equipo",
      openFolder: "No se pudo abrir la carpeta",
      archive: "No se pudo archivar el equipo",
    },
  },
  bots: {
    empty: (crew: string) => `Aún no hay bots en ${crew}.`,
    emptyHint:
      "Un bot es una sesión de Claude Code que sigue en marcha, con su propia carpeta y su rol.",
    noRole: "Aún sin rol.",
  },
  tasks: {
    show: "Mostrar",
    open: "Abiertas",
    all: "Todas",
    list: "Tareas",
    noOpen: "No hay tareas abiertas. Los bots se crean tareas entre sí con send_message.",
    none: "Aún no hay tareas. Los bots se crean tareas entre sí con send_message.",
    status: {
      open: "Abierta",
      done: "Completada",
      failed: "Falló",
      cancelled: "Cancelada",
      expired: "Vencida",
    },
    archivedBot: "un bot archivado",
    asked: "le pidió a",
    hop: (hops: number) => `paso ${hops}`,
    hopHint: "Posición en una cadena de delegaciones",
    due: (time: string) => `vence ${time}`,
    overdue: (time: string) => `atrasada, venció ${time}`,
    ended: (status: string, time: string) => `${status.toLowerCase()} ${time}`,
  },
};
