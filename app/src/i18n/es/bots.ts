import type { Messages } from "../en";

export const bots: Messages["bots"] = {
  states: {
    offline: { label: "Detenido", hint: "No está en ejecución." },
    launching: { label: "Iniciando", hint: "Claude Code se está iniciando." },
    idle: { label: "Disponible", hint: "Listo para trabajar." },
    busy: { label: "Trabajando", hint: "Trabajando en algo." },
    needs_approval: {
      label: "Necesita aprobación",
      hint: "Esperando a que permitas o deniegues una herramienta en su chat.",
    },
    rate_limited: {
      label: "Límite de uso",
      hint: "Tu plan de Claude alcanzó su límite de uso; los mensajes esperan hasta que se restablezca.",
    },
    auth_error: {
      label: "Necesita iniciar sesión",
      hint: "Claude Code no tiene una sesión iniciada, o la cuenta no se puede usar. Inicia sesión en Claude y el bot vuelve a iniciarse solo. ¿Ya iniciaste sesión? Revisa tu plan de Claude y luego reinicia el bot.",
    },
    backoff: {
      label: "Reiniciando",
      hint: "Se detuvo de forma inesperada; Botloft lo vuelve a iniciar en breve.",
    },
    archived: { label: "Archivado", hint: "Archivado." },
    paused: { label: "En pausa", hint: "En pausa; reanúdalo para iniciarlo." },
  },
  header: {
    noRole: "Sin rol",
    pause: "Pausar",
    resume: "Reanudar",
    restart: "Reiniciar",
    showDetails: "Mostrar detalles",
    hideDetails: "Ocultar detalles",
    more: "Más acciones del bot",
    edit: "Editar",
    restartFresh: "Reiniciar con una conversación nueva",
    openFolder: "Abrir carpeta",
    archive: "Archivar bot",
    failed: {
      pause: "No se pudo pausar el bot",
      resume: "No se pudo reanudar el bot",
      restart: "No se pudo reiniciar el bot",
      openFolder: "No se pudo abrir la carpeta",
      archive: "No se pudo archivar el bot",
    },
    fresh: {
      title: "¿Empezar una conversación nueva?",
      confirm: "Reiniciar",
      body: (name: string) =>
        `${name} se reinicia sin su conversación actual. Su carpeta y su CLAUDE.md se quedan como están.`,
    },
    archiveConfirm: {
      title: (name: string) => `¿Archivar ${name}?`,
      confirm: "Archivar bot",
      body: "El bot se detiene y sale del equipo. Los mensajes que aún lo esperan no se entregan.",
    },
  },
  dialog: {
    newTitle: "Nuevo bot",
    editTitle: (name: string) => `Editar ${name}`,
    create: "Crear bot",
    save: "Guardar",
    name: "Nombre",
    namePlaceholder: "Revisor",
    nameHint: "Los otros bots lo llaman por el @ que se forma con este nombre.",
    role: "Rol",
    rolePlaceholder: "Revisa los pull requests antes de fusionarlos",
    instructions: "Instrucciones",
    instructionsPlaceholder:
      "Cómo trabaja este bot, qué puede hacer por su cuenta y cuándo debe preguntar.",
    instructionsHint:
      "Se guardan ahora en las reglas del bot; el bot las lee la próxima vez que se inicie.",
    color: "Color",
    swatch: (color: string) => `Color ${color}`,
    colorUnset: "Si no eliges uno, el bot recibe el siguiente color del equipo.",
  },
  notices: {
    crewPaused: (crew: string) => `El equipo ${crew} está en pausa`,
    crewPausedBody: "Sus bots siguen detenidos hasta que reanudes el equipo.",
  },
  details: {
    title: (name: string) => `Acerca de ${name}`,
    close: "Cerrar detalles",
    role: "Rol",
    noRole: "Todavía sin rol.",
    folder: "Carpeta",
    process: "Proceso",
    notStarted: "no iniciado",
    generation: (generation: number) => `generación ${generation}`,
    instructions: "Instrucciones",
    noInstructions: "Ninguna todavía.",
  },
};
