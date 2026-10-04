import type { Messages } from "../en";

export const desktop: Messages["desktop"] = {
  card: {
    wants: (bot) => `${bot} quiere ver`,
    asks: (bot, app) => `${bot} pide ver ${app}`,
    because: (bot) => `${bot} dice:`,
    means: (bot) =>
      `${bot} va a leer las ventanas de esa app y sacarles fotos mientras estés en la computadora. Nunca tus contraseñas. Puedes quitárselo en los detalles de ${bot}.`,
    allowed: (bot, app) => `${bot} puede ver ${app}`,
    denied: (bot, app) => `${bot} no puede ver ${app}`,
    expired: (app) => `Sin respuesta sobre ${app}`,
  },
  grants: {
    title: "Tu escritorio",
    none: (bot) =>
      `${bot} no ve ninguna app de tu computadora. La pide en el chat la primera vez que la necesita.`,
    see: "Puede ver",
    act: "Puede ver y usar",
    whole: "Todo el escritorio",
    remove: (app) => `Quitar ${app}`,
    removeFailed: "No se pudo quitar",
  },
};
