// Claves de los agentes (spec 32).

import type { Messages } from "../en";

export const keys: Messages["keys"] = {
  card: {
    asks: (bot: string) => `${bot} necesita una clave`,
    why: "Para qué",
    saved: (name: string) => `Guardaste la clave ${name}`,
    dismissed: (name: string) => `No diste la clave ${name}`,
    notNow: "Ahora no",
    notNowHint: (bot: string) => `Cierra el pedido y avisa a ${bot} que la clave no llega ahora.`,
    dismissFailed: "No se pudo cerrar el pedido",
  },
  form: {
    name: "Nombre",
    nameHint: "Mayúsculas, números y _, como GITHUB_TOKEN. El agente lee la clave por él.",
    value: "Clave",
    valueHint:
      "Queda solo en esta computadora, fuera de la conversación. Nadie puede leerla de vuelta aquí.",
    description: "Descripción (opcional)",
    descriptionPlaceholder: "Para qué sirve",
    show: "Mostrar la clave",
    hide: "Ocultar la clave",
    save: "Guardar",
    cancel: "Cancelar",
    saveFailed: "No se pudo guardar la clave",
    restarts: (bot: string) => `${bot} se reinicia para recibirla, sin perder la conversación.`,
  },
  details: {
    title: "Claves",
    none: (bot: string) =>
      `Ninguna clave todavía. Cuando ${bot} necesite un token o una contraseña de otro servicio, guárdala aquí en vez de pegarla en el chat.`,
    howItWorks: "Cómo funciona",
    explain: (bot: string) =>
      `${bot} recibe cada clave como una variable con su nombre, y Botloft cambia por el nombre una clave que aparece en el chat. No es una garantía: si ${bot} imprime la clave de todos modos, queda en el historial del propio motor. La mejor protección es una clave que puede poco.`,
    add: "Agregar clave",
    replace: (name: string) => `Cambiar ${name}`,
    remove: (name: string) => `Borrar ${name}`,
    missing: "Falta el valor",
    missingHint:
      "El nombre vino con una copia de seguridad, el valor no. Cámbiala para dar la clave de nuevo.",
    changed: (when: string) => `cambió ${when}`,
    removeTitle: (name: string) => `¿Borrar la clave ${name}?`,
    removeBody: (bot: string) =>
      `${bot} se reinicia sin ella. La clave sigue valiendo en el servicio de donde vino: para anularla allí, revócala en ese servicio.`,
    removeConfirm: "Borrar",
    removeFailed: "No se pudo borrar la clave",
  },
  pasted: (names: string[], bot: string) =>
    `${names.length === 1 ? `La clave ${names[0]} ya está guardada` : `Las claves ${names.join(", ")} ya están guardadas`}, así que ${bot} recibió el nombre en lugar del valor.`,
};
