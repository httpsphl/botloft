import type { Messages } from "../en";

export const shell: Messages["shell"] = {
  window: {
    minimize: "Minimizar",
    maximize: "Maximizar",
    restore: "Restaurar",
    close: "Cerrar",
  },
  zoom: {
    level: (percent: number, isDefault: boolean) =>
      isDefault ? `${percent}% (predeterminado)` : `${percent}%`,
  },
  language: {
    label: "Idioma",
    system: (name: string) => `Idioma del sistema: ${name}`,
  },
  connection: {
    lost: "Desconectado",
    reconnecting: "Reconectando…",
  },
  loadFailed: "No se pudieron cargar tus equipos",
  pickCrew: "Elige un equipo a la izquierda, o crea uno con el botón +.",
  botsCantStart: "Los bots no pueden iniciar",
  signIn: {
    title: "Inicia sesión en Claude",
    body: "Tus bots trabajan con tu cuenta de Claude. Inicia sesión una vez y empiezan solos.",
  },
};
