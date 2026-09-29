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
  markdown: {
    image: "imagen",
    openLinkFailed: "No se pudo abrir el enlace",
  },
  notice: {
    signedOut:
      "Claude Code no tiene una sesión iniciada, o la cuenta no se puede usar ahora. Inicia sesión en Claude o revisa tu plan de Claude.",
    usageLimit:
      "Tu plan de Claude alcanzó su límite de uso. Los mensajes esperarán hasta que se restablezca.",
    turnFailed: (detail: string) => `El bot no pudo terminar esto: ${detail}`,
  },
};
