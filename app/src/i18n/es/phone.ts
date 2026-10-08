// La página del móvil (spec 28.7).

import type { Messages } from "../en";

export const phone: Messages["phone"] = {
  pair: {
    title: "Conectar este móvil",
    intro:
      "Este móvil aprobará las solicitudes de tus bots y responderá a sus preguntas, desde cualquier lugar. Las conversaciones y los archivos se quedan en tu ordenador.",
    nameLabel: "Nombre de este móvil",
    connect: "Conectar",
    connecting: "Conectando…",
    compare: (code: string) =>
      `Comprueba que el ordenador muestre ${code} también. Luego pulsa Conectar en el ordenador.`,
    waiting: "Esperando a que aceptes en el ordenador…",
    notALink:
      "Para conectar un móvil, apunta su cámara al código que aparece en el ordenador: Ajustes, Copia de seguridad, Móvil.",
    expired: "El código venció o ya se usó. Pide uno nuevo en el ordenador.",
    offline: "No se pudo hablar con Botloft. Comprueba la conexión e inténtalo de nuevo.",
    failed: "No se pudo conectar. Pide un código nuevo en el ordenador.",
    names: { iphone: "iPhone", ipad: "iPad", android: "Móvil Android", other: "Móvil" },
  },
  inbox: {
    title: "Esperándote",
    empty: "Nada te está esperando",
    emptyBody: "Cuando un bot te necesite, aparecerá aquí.",
    loading: "Viendo qué está esperando…",
    computerOff: "El ordenador está apagado o sin internet. Las solicitudes siguen esperando allí.",
    noConnection: "Sin conexión. Intentando de nuevo…",
    thisPhone: "Este móvil",
    inCrew: (crew: string) => `en ${crew}`,
  },
  approval: {
    asks: (bot: string) => `${bot} pide`,
    explanationBy: (bot: string) => `${bot} escribió esto`,
    noExplanation: (bot: string) => `${bot} no dijo para qué sirve. Conviene denegar y preguntar.`,
    showAll: "Mostrar la solicitud entera",
    hideAll: "Ocultar",
    allow: "Permitir",
    deny: "Denegar",
    sending: "Enviando…",
    noteLabel: "Nota para el bot (opcional)",
    cut: "Demasiado larga para leerla en el móvil. Aquí solo puedes denegarla, o responderla en el ordenador.",
    atComputer: "Esta solicitud te necesita en el ordenador. Aquí solo puedes denegarla.",
    failed: "No se envió. Comprueba la conexión e inténtalo de nuevo.",
    ended: { allowed: "Permitido", denied: "Denegado", expired: "Venció" },
  },
  question: {
    asks: (bot: string) => `${bot} pregunta`,
    pick: "Elige una respuesta",
    answerLabel: "Tu respuesta",
    placeholder: "Escribe tu respuesta",
    send: "Responder",
    sending: "Enviando…",
    dismiss: "Descartar",
    dismissHint: (bot: string) =>
      `Cierra la pregunta sin avisar a ${bot}. Para que lo sepa, responde.`,
    failed: "No se envió. Comprueba la conexión e inténtalo de nuevo.",
    ended: { answered: "Respondiste", dismissed: "Descartada" },
  },
  settings: {
    notices: {
      turnOn: "Activar los avisos",
      turnOff: "Desactivar los avisos",
      on: "Los avisos están activados. Recibes uno cuando un bot te necesite.",
      off: "Los avisos están desactivados.",
      blocked:
        "Los avisos están bloqueados para esta página. Permítelos en los ajustes del navegador.",
      unsupported: "Este navegador no puede dar avisos.",
    },
    title: "Este móvil",
    back: "Volver",
    name: "Nombre",
    disconnect: "Desconectar este móvil",
    disconnectTitle: "¿Desconectar este móvil?",
    disconnectText:
      "Deja de recibir solicitudes. Para usarlo de nuevo, conéctalo otra vez en el ordenador.",
    cancel: "Cancelar",
    install:
      "Para recibir avisos, añade Botloft a la pantalla de inicio: toca Compartir y luego Añadir a pantalla de inicio.",
  },
  cut: {
    revokedTitle: "Este móvil fue desconectado",
    leftTitle: "Móvil desconectado",
    body: "Para usarlo de nuevo, conéctalo desde Botloft en el ordenador: Ajustes, Copia de seguridad, Móvil.",
    ok: "OK",
  },
};
