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
    folder: "Carpeta de trabajo",
    folderHint: "Donde los bots guardan lo que hacen. Puede ser una carpeta que ya usas.",
    folderDefault: "Una carpeta nueva dentro de Botloft",
    chooseFolder: "Elegir carpeta…",
    pickTitle: "Elige dónde trabaja el equipo",
    useDefault: "Usar una carpeta nueva",
    goal: "¿Para qué es este equipo?",
    goalPlaceholder: "Crear y mantener la web de mi panadería",
    goalHint:
      "El equipo empieza con un jefe, que lee esto, planea el trabajo y sugiere los bots que necesita.",
    chiefModel: "Modelo del jefe",
    chiefName: "Jefe",
    chiefRole: "Dirige el equipo: planea el trabajo, sugiere bots nuevos y reparte las tareas",
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
    folder: (path: string) => `Trabaja en ${path}`,
    openFolder: "Abrir carpeta de trabajo",
    changeFolder: "Cambiar carpeta de trabajo…",
    moveTitle: (crew: string) => `¿Mover ${crew} a otra carpeta?`,
    moveBody: (path: string) =>
      `Los bots trabajarán en ${path}. Cada uno se reinicia cuando termine lo que está haciendo. Lo que ya hicieron se queda donde está.`,
    move: "Mover",
    archive: "Archivar equipo",
    delete: "Eliminar equipo",
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
    deleteTitle: (crew: string) => `¿Eliminar ${crew}?`,
    deleteBody: (crew: string, bots: number) =>
      bots === 0
        ? `${crew} sale de Botloft para siempre. No se puede deshacer.`
        : `${crew} y ${bots === 1 ? "su bot" : `sus ${bots} bots`} se detienen ahora y salen de Botloft para siempre, con sus conversaciones, rutinas y tareas. No se puede deshacer.`,
    deleteKept:
      "Las carpetas siguen en tu ordenador, con todo lo que contienen: la carpeta de cada bot y la carpeta de trabajo del equipo:",
    failed: {
      pause: "No se pudo pausar el equipo",
      resume: "No se pudo reanudar el equipo",
      changeFolder: "No se pudo cambiar la carpeta",
      openFolder: "No se pudo abrir la carpeta",
      archive: "No se pudo archivar el equipo",
      delete: "No se pudo eliminar el equipo",
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
