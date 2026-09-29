import type { Messages } from "../en";

export const chat: Messages["chat"] = {
  view: {
    label: (bot: string) => `Chat con ${bot}`,
    loadFailed: "No se pudo cargar el chat",
    loadEarlier: "Cargar mensajes anteriores",
    loading: "Cargando el chat…",
    emptyTitle: (bot: string) => `Empieza una conversación con ${bot}`,
    emptyBody:
      "Pídele lo que su carpeta y sus herramientas le permitan hacer. También puedes adjuntar archivos e imágenes.",
    messages: "Mensajes",
    dropToAttach: "Suelta para adjuntar a tu mensaje",
  },
  composer: {
    paused: (bot: string) =>
      `${bot} está en pausa. Lo que envíes esperará hasta que vuelva a funcionar.`,
    filesToSend: "Archivos para enviar",
    remove: (file: string) => `Quitar ${file}`,
    label: (bot: string) => `Mensaje para ${bot}`,
    placeholder: (bot: string) => `Escribe a ${bot}`,
    attach: "Adjuntar archivos",
    attachHint: "Adjuntar archivos (o pégalos, o suéltalos en el chat)",
    tooLong: (length: number, max: number) => `${length}/${max} caracteres`,
    keys: "Enter para enviar, Shift+Enter para una nueva línea",
    send: "Enviar",
  },
  files: {
    tooMany: (max: number) => `Un mensaje puede llevar hasta ${max} archivos.`,
    tooBig: (size: string) =>
      `Los archivos de un mensaje pueden sumar hasta ${size}. Envíalos en partes.`,
    unreadable: (file: string) => `no se pudo leer ${file}`,
  },
  attachments: {
    label: "Adjuntos",
    file: "Archivo",
    showInFolder: (file: string) => `Mostrar ${file} en su carpeta`,
    showInFolderHint: "Mostrar en la carpeta",
    openFolderFailed: "No se pudo abrir la carpeta",
    missing: "Ya no está en la carpeta del bot",
    loading: (file: string) => `Cargando ${file}`,
  },
  inbound: {
    task: "Tarea",
    result: "Resultado",
    archivedBot: "Un bot archivado",
  },
  run: {
    working: "Trabajando",
    done: (time: string) => `Terminado en ${time}`,
    took: (time: string, usd: number | null) =>
      `Tardó ${time}${
        usd === null
          ? ""
          : ` · unos ${usd.toLocaleString("es", { style: "currency", currency: "USD" })} de uso`
      }`,
    stopped: (reason: string) => `El bot dejó de trabajar en esto: ${reason}`,
  },
  tools: {
    label: "Llamadas a herramientas",
    running: "En curso",
    done: "Listo",
    failed: "Falló",
    input: "Entrada",
    output: "Salida",
    error: "Error",
  },
  approval: {
    asks: (bot: string, tool: string) => `${bot} pide usar ${tool}`,
    wants: (bot: string, tool: string) => `${bot} quiere usar ${tool}`,
    fullInput: "Entrada completa",
    noteLabel: (bot: string) => `Nota para ${bot} si deniegas`,
    notePlaceholder: "¿Por qué no? Se envía al bot si deniegas (opcional)",
    allow: "Permitir",
    deny: "Denegar",
    allowFailed: "No se pudo permitir",
    denyFailed: "No se pudo denegar",
    allowed: (tool: string) => `Permitiste ${tool}`,
    denied: (tool: string) => `Denegaste ${tool}`,
    expired: (tool: string) => `${tool} no se aprobó a tiempo`,
  },
  mode: {
    title: "Modo",
    button: (mode: string) => `Modo: ${mode}`,
    names: {
      auto: "Automático",
      default: "Manual",
      accept_edits: "Aceptar ediciones",
      plan: "Plan",
      bypass_permissions: "Omitir permisos",
    },
    hints: {
      auto: (bot: string) => `${bot} decide qué necesita tu aprobación`,
      default: (bot: string) => `${bot} siempre pregunta antes de hacer cambios`,
      accept_edits: (bot: string) => `${bot} edita archivos sin preguntar`,
      plan: (bot: string) => `${bot} crea un plan antes de hacer cambios`,
      bypass_permissions: (bot: string) => `${bot} lo hace todo sin preguntar`,
    },
    turnOn: "Activar",
    failed: "No se pudo cambiar el modo",
    later: (bot: string, mode: string) =>
      `${bot} cambiará a ${mode} cuando termine lo que está haciendo.`,
    bypassTitle: (bot: string) => `¿Dejar que ${bot} haga todo sin preguntar?`,
    bypassBody: (bot: string) =>
      `${bot} editará archivos, ejecutará comandos y usará internet en este equipo sin pedirte permiso antes.`,
    bypassRisk:
      "No se limita a su carpeta: puede leer y cambiar tus otros archivos, los de otros bots y los del propio Botloft. Un mensaje de otra persona podría llevarlo a hacer algo que no querías.",
    bypassAdvice:
      "Actívalo solo para un bot en el que confíes para todo, y solo mientras lo necesites.",
    badge: "No pregunta nada",
    badgeHint:
      "Este bot lo hace todo sin preguntar. Cámbialo en el selector de modo, debajo del chat.",
  },
  model: {
    title: "Modelo",
    button: (model: string) => `Modelo: ${model}`,
    names: {
      default: "Predeterminado del plan",
      fable: "Fable",
      opus: "Opus",
      sonnet: "Sonnet",
      haiku: "Haiku",
    },
    short: "Predeterminado",
    hints: {
      default: (bot: string, inUse: string | null) =>
        inUse
          ? `${bot} usa el modelo predeterminado de tu plan, ahora ${inUse}`
          : `${bot} usa el modelo predeterminado de tu plan`,
      fable: (bot: string) => `${bot} rinde al máximo, para el trabajo más difícil`,
      opus: (bot: string) => `${bot} maneja bien tareas largas y complejas`,
      sonnet: (bot: string) => `${bot} es rápido y capaz, bueno para casi todo`,
      haiku: (bot: string) =>
        `${bot} es el más rápido y gasta menos de tu plan, para tareas simples`,
    },
    cost: "Los modelos más capaces gastan el límite de tu plan más rápido.",
    failed: "No se pudo cambiar el modelo",
    later: (bot: string, model: string) =>
      `${bot} cambia a ${model} cuando termine lo que está haciendo.`,
  },
  plan: {
    ready: (bot: string) => `${bot} hizo un plan y quiere seguir con él`,
    noteLabel: (bot: string) => `Qué debe cambiar ${bot} en el plan`,
    notePlaceholder: "¿Qué debe cambiar? Se envía al bot si pides cambios (opcional)",
    approve: "Aprobar plan",
    keepPlanning: "Pedir cambios",
    approveFailed: "No se pudo aprobar el plan",
    keepFailed: "No se pudo devolver el plan",
    approved: "Aprobaste el plan",
    sentBack: "Pediste cambios en el plan",
    expired: "El plan no se aprobó a tiempo",
  },
  markdown: {
    image: "imagen",
    openLinkFailed: "No se pudo abrir el enlace",
  },
  notice: {
    signedOut:
      "Claude Code no tiene una sesión iniciada, o la cuenta no se puede usar ahora. Inicia sesión en Claude o revisa tu plan de Claude.",
    usageLimit:
      "Tu plan de Claude alcanzó su límite de uso. Los mensajes esperarán hasta que se restablezca.",
    modelUnavailable:
      "El modelo de este bot no está disponible: quizá no esté en tu plan de Claude. Elige otro modelo debajo del chat.",
    turnFailed: (detail: string) => `El bot no pudo terminar esto: ${detail}`,
  },
};
