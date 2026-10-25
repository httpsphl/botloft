// La isla en la parte de arriba de la pantalla (spec 31).

import type { Messages } from "../en";

export const island: Messages["island"] = {
  label: "Botloft en la parte de arriba de la pantalla",
  waiting: (count) => (count === 1 ? "1 esperando" : `${count} esperando`),
  working: (count) => (count === 1 ? "1 trabajando" : `${count} trabajando`),
  position: (at, of) => `${at} de ${of}`,
  previous: "Anterior",
  next: "Siguiente",
  tabs: {
    label: "Pestañas de la isla",
    home: "Inicio",
  },
  openApp: "Abrir Botloft",
  hide: "Ocultar",
  engines: { claude: "Claude Code", agy: "Antigravity", codex: "Codex" },
  thinking: "Pensando",
  moreWorking: (count) => (count === 1 ? "+1 trabajando" : `+${count} trabajando`),
  finished: (bot) => `${bot} terminó`,
  today: {
    title: "Hoy",
    since: "desde la medianoche",
    tasks: (count) => (count === 1 ? "tarea" : "tareas"),
    requests: (count) => (count === 1 ? "solicitud" : "solicitudes"),
    lines: (files) => (files === 1 ? "líneas, 1 archivo" : `líneas, ${files} archivos`),
    busiest: "trabajó más",
  },
  card: {
    allow: "Permitir",
    deny: "Denegar",
    openInApp: "Abrir en Botloft",
    cut: "Demasiado largo para mostrarse entero aquí. Ábrelo en Botloft para leerlo todo.",
    atComputer: "Te necesita en Botloft",
    asks: (bot) => `${bot} pregunta`,
    answerLabel: (bot) => `Tu respuesta para ${bot}`,
    placeholder: "Escribe tu respuesta",
    send: "Responder",
    failed: "No se pudo enviar. Inténtalo de nuevo, o ábrelo en Botloft.",
  },
};
