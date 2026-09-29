// El icono junto al reloj y los avisos de Windows (spec 15.1).

import type { Messages } from "../en";

export const alerts: Messages["alerts"] = {
  tray: {
    tooltip: (status) => `Botloft: ${status}`,
    working: (count) => (count === 1 ? "1 bot trabajando" : `${count} bots trabajando`),
    needsYou: (bot) => `${bot} te necesita`,
    waiting: "Algo te está esperando",
    idle: "Ningún bot trabajando",
    open: "Abrir Botloft",
    pause: "Pausar todos los equipos",
    resume: "Reanudar todos los equipos",
    quit: "Salir de Botloft",
  },
  approval: (bot) => `${bot} pide tu permiso`,
  signIn: (bot) => `${bot} necesita que inicies sesión en Claude`,
  signInBody: "Abre Botloft para iniciar sesión.",
  done: (bot) => `${bot} terminó`,
  crew: (crew) => `Equipo ${crew}`,
};
