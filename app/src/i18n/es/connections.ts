import type { Messages } from "../en";

export const connections: Messages["connections"] = {
  title: "Herramientas conectadas",
  intro:
    "Herramientas tuyas que tus agentes pueden usar además de las que Botloft ya les da, como un lector de LinkedIn o un sistema de tu empresa. Cada agente recibe solo las que activas para él, y aun así te pregunta antes de usarlas.",
  loadFailed: "No se pudieron leer las herramientas conectadas",
  none: "Aún no hay herramientas conectadas.",
  add: "Conectar una herramienta…",
  usedBy: (names) => `La usan ${names}`,
  unused: "Ningún agente la usa todavía",
  kinds: {
    stdio: "Un programa en este equipo",
    http: "Una herramienta en una dirección",
  },
  edit: (name) => `Editar ${name}`,
  remove: (name) => `Quitar ${name}`,
  adding: {
    title: "Conectar una herramienta",
    paste: "Pega su configuración",
    pasteHint: "El texto de su archivo .mcp.json, o la configuración que vino con la herramienta.",
    name: "Nombre",
    nameHint: "Solo hace falta cuando la configuración pegada viene sin nombre.",
    purpose: "¿Para qué sirve? (opcional)",
    purposeHint: "Tus agentes leen esto para saber cuándo usarla.",
    found: (count) => (count === 1 ? "Encontré 1 herramienta." : `Encontré ${count} herramientas.`),
    program: (name) =>
      `${name} es un programa. Se ejecutará en tu equipo con tus permisos, fuera de las carpetas que Botloft guarda para tus agentes. Conecta solo programas en los que confíes.`,
    address: (name) =>
      `${name} es una herramienta en una dirección. Lo que tu agente le envíe sale de este equipo.`,
    command: "Lo que ejecuta",
    understood: "Lo entiendo y confío en ella",
    connect: "Conectar",
    connecting: "Conectando…",
    failed: "No se pudo conectar la herramienta",
  },
  problems: {
    empty: "Pega primero la configuración de la herramienta.",
    notJson: "Eso no es un JSON válido. Pega el texto tal como está en el archivo.",
    noServers: "No hay ninguna herramienta en este texto.",
    needsName: "Esta configuración no tiene nombre: escribe uno abajo.",
    unsupported: (server) => `${server} usa un tipo de conexión que Botloft aún no admite.`,
    unknownField: (server, field) =>
      `${server} tiene un ajuste que Botloft no conoce: "${field}". Quítalo e inténtalo de nuevo.`,
    badValue: (server, field) =>
      `${server} tiene algo mal en "${field}". Revísalo e inténtalo de nuevo.`,
  },
  editing: {
    title: (name) => `Editar ${name}`,
    note: "Para cambiar lo que ejecuta, su dirección o sus contraseñas, quítala y conéctala de nuevo.",
    save: "Guardar",
    failed: "No se pudo guardar la herramienta",
  },
  removing: {
    title: (name) => `¿Quitar ${name}?`,
    text: "Los agentes que la usan la pierden y arrancan de nuevo cuando estén libres. Lo que permitiste para siempre con ella se olvida.",
    confirm: "Quitar",
    failed: "No se pudo quitar la herramienta",
  },
  bot: {
    title: "Herramientas conectadas",
    none: "Aún no hay herramientas conectadas. Puedes conectarlas en Ajustes, en Herramientas conectadas.",
    toggle: (tool, bot) => `${bot} puede usar ${tool}`,
    failed: "No se pudieron cambiar las herramientas de este agente",
    reason: "Por qué",
  },
  states: {
    connected: "Conectada",
    pending: "Conectando…",
    needs_auth: "Necesita que inicies sesión",
    failed: "No conectó",
  },
};
