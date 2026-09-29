import type { Messages } from "../en";

export const common: Messages["common"] = {
  cancel: "Cancelar",
  close: "Cerrar",
  resize: "Cambiar el tamaño del panel",
  dismiss: "Descartar",
  details: "Detalles",
  tryAgain: "Reintentar",
  errors: {
    closed: "la conexión se cerró",
    notConnected: "sin conexión con Botloft",
    connectionLost: "se perdió la conexión con Botloft",
    timedOut: (method: string) => `${method} tardó demasiado`,
  },
};
