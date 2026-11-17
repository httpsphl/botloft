import type { Messages } from "../en";

export const home: Messages["home"] = {
  label: "Inicio",
  title: "Inicio",
  summary: (waiting: number, working: number) =>
    [
      waiting > 0 && `${waiting} ${waiting === 1 ? "cosa te espera" : "cosas te esperan"}`,
      working > 0 && `${working} trabajando`,
    ]
      .filter(Boolean)
      .join(" · ") || "Todo tranquilo",
  needsYou: "Te necesita",
  working: "Trabajando ahora",
  replies: "Respuestas nuevas",
  crews: "Crews",
  today: {
    title: "Hoy",
    nothing: "Nada hecho todavía hoy.",
    tasks: "Tareas hechas",
    failed: (count: number) => `${count} ${count === 1 ? "falló" : "fallaron"}`,
    requests: "Solicitudes respondidas",
    files: "Archivos editados",
    messages: "Mensajes entre agentes",
    busiest: "Más trabajó",
  },
  empty: {
    title: "Nada te espera",
    body: "Cuando un agente pida tu permiso, una decisión o una clave, aparece aquí, de todas las crews.",
  },
  request: {
    seeAll: "Ver la solicitud",
    openChat: (bot: string) => `Abrir el chat con ${bot}`,
    openInChat: "Abrir en el chat",
    cut: "Demasiado larga para mostrarla entera aquí. Para permitirla, ábrela en el chat.",
    atComputer: "Esta cambia las cosas para siempre, así que la respondes en el chat.",
    gone: "Ya se respondió",
  },
  key: {
    title: "Necesita una clave",
    inChat: "Pegas la clave en el chat, en una tarjeta que la guarda fuera de la conversación.",
  },
  failed: {
    line: (count: number) =>
      count === 1 ? "1 mensaje no se entregó" : `${count} mensajes no se entregaron`,
    show: "Ver y reintentar",
  },
  signedOut: {
    line: (bot: string) => `${bot} se detuvo: su motor no tiene la sesión iniciada`,
    open: "Abrir",
  },
};
