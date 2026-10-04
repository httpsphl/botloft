import type { Messages } from "../en";

export const shell: Messages["shell"] = {
  window: {
    minimize: "Minimizar",
    maximize: "Maximizar",
    restore: "Restaurar",
    close: "Cerrar",
  },
  /** The button that hides the crews and bots on the left (Ctrl+B). */
  rail: {
    label: "Lugares",
    home: "Inicio",
  },
  sidebar: {
    hide: "Ocultar la lista de bots",
    show: "Mostrar la lista de bots",
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
  recycleFailed: (path: string) => `La carpeta ${path} no fue a la Papelera y sigue allí`,
  botsCantStart: "Los bots no pueden iniciar",
  signIn: {
    title: "Inicia sesión en Claude",
    body: "Tus bots trabajan con tu cuenta de Claude. Inicia sesión una vez y empiezan solos.",
  },
};
