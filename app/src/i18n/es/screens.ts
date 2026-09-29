// El área de diseño (spec 22.5): las pantallas HTML que hace el bot, en
// vivo junto al chat mientras las escribe.

import type { Messages } from "../en";

export const screens: Messages["screens"] = {
  heading: "Pantallas",
  panel: (bot) => `Pantallas de ${bot}`,
  show: "Mostrar pantallas",
  hide: "Ocultar pantallas",
  drawing: (bot) => `${bot} está dibujando una pantalla`,
  showInPanel: "Ver en pantallas",
  close: "Cerrar",
  expand: "Ensanchar",
  shrink: "Estrechar",
  zoomIn: "Acercar",
  zoomOut: "Alejar",
  fit: "Ajustar",
  board: "Pantallas",
  open: (name) => `Abrir ${name}`,
  writing: "Escribiendo…",
  back: "Todas las pantallas",
  openFile: "Abrir",
  reveal: "Mostrar en la carpeta",
  device: "Dispositivo",
  devices: {
    desktop: "Computadora",
    tablet: "Tableta",
    mobile: "Celular",
  },
  more: (count) => `Mostrar ${count} más`,
  emptyTitle: (bot) => `${bot} todavía no hizo ninguna pantalla`,
  emptyBody: "Cada página HTML que escriba aparece aquí, y la ves tomar forma mientras la escribe.",
  loadFailed: "No se pudieron cargar las pantallas",
  gone: "Esta pantalla ya no está.",
  failed: {
    open: "No se pudo abrir la pantalla",
    reveal: "No se pudo mostrar el archivo",
  },
};
