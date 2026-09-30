import type { Messages } from "../en";

export const messages: Messages["messages"] = {
  composer: {
    messageTo: "Mensaje para",
    recipient: "Destinatario",
    placeholder: (handle: string | undefined) => `Escribe a @${handle ?? "bot"}. Ctrl+Enter envía.`,
    send: "Enviar",
  },
  delivery: {
    delivered: "Entregado",
    read: "Leído",
    readTitle: "El bot empezó a trabajar en él",
    delivering: "Entregando",
    waiting: "Esperando al bot",
    retrying: (when: string, attempts: number) =>
      `Nuevo intento ${when}, tras ${attempts} ${attempts === 1 ? "intento fallido" : "intentos fallidos"}`,
    notDelivered: "No entregado",
    retry: "Reintentar",
    retryFailed: "No se pudo reintentar la entrega",
  },
  failed: {
    button: (count: number) => `${count} no ${count === 1 ? "entregado" : "entregados"}`,
    title: "Mensajes no entregados",
    retryAll: "Reintentar todos",
    explanation:
      "Botloft dejó de insistir tras varios intentos. Al reintentar, el mensaje vuelve a la cola; se envía cuando el bot esté listo.",
    to: (name: string) => `Para ${name}`,
    tries: (count: number) => `${count} ${count === 1 ? "intento" : "intentos"}`,
  },
  row: {
    you: "Tú",
    system: "Botloft",
    goneBot: "un bot que ya no está en el equipo",
    to: "para",
    task: "Tarea",
    result: "Resultado",
    due: (when: string) => `vence ${when}`,
    taskStatus: {
      open: "abierta",
      done: "completada",
      failed: "fallida",
      cancelled: "cancelada",
      expired: "vencida",
    },
  },
  timeline: {
    list: "Mensajes",
    loadFailed: "No se pudieron cargar los mensajes",
    loadOlder: "Cargar mensajes anteriores",
    loading: "Cargando mensajes…",
  },
};
