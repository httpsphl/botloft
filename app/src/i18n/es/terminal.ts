import type { Messages } from "../en";

export const terminal: Messages["terminal"] = {
  heading: "Terminal",
  panel: (bot) => `Terminal de ${bot}`,
  title: (bot) => `${bot} — comandos`,
  close: "Cerrar",
  showInPanel: "Ver en la terminal",
  running: "ejecutando…",
  loading: "Cargando los comandos…",
  empty: (bot) => `${bot} aún no ejecutó ningún comando. Lo que ejecute aparece aquí, en vivo.`,
  dock: {
    label: "Computadora",
    browser: "Navegador",
    terminal: "Terminal",
    files: "Archivos",
    desktop: "Escritorio",
  },
};
