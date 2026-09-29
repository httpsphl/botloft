// El navegador de cada bot (spec 21.8): el panel junto al chat donde el
// dueño lo mira en vivo, y la tarjeta en la que pide usar un sitio nuevo.

import type { Messages } from "../en";

export const browser: Messages["browser"] = {
  heading: "Navegador",
  panel: (bot) => `Navegador de ${bot}`,
  show: "Mostrar navegador",
  hide: "Ocultar navegador",
  browsing: (bot) => `${bot} está usando el navegador`,
  showInPanel: "Ver en el navegador",
  close: "Cerrar",
  expand: "Ensanchar",
  shrink: "Estrechar",
  openOutside: "Abrir en mi navegador",
  openFailed: "No se pudo abrir la página",
  live: "En vivo",
  loading: "Cargando…",
  address: "Dirección",
  screen: (bot) => `Lo que ve ${bot}`,
  emptyTitle: (bot) => `${bot} todavía no abrió el navegador`,
  emptyBody:
    "Cuando busque o use un sitio, lo verás aquí en vivo: cada página que abre, cada clic.",
  starting: "Abriendo el navegador…",
  closed: "Navegador cerrado",
  closedBody: "Se abre de nuevo cuando el bot lo necesite. Las sesiones iniciadas siguen.",
  failedTitle: "El navegador no se abrió",
  failedBody:
    "Botloft usa Microsoft Edge, que viene con Windows. Comprueba que esté instalado y pide al bot que lo intente de nuevo.",
  details: "Detalles",
  tabs: (count) => (count === 1 ? "1 pestaña" : `${count} pestañas`),
  did: {
    open: (site) => `Abrió ${site}`,
    click: (what) => `Hizo clic en ${what}`,
    clickSomewhere: "Hizo clic",
    type: (what) => `Escribió en ${what}`,
    typeSomewhere: "Escribió",
    select: (what) => `Eligió en ${what}`,
    press: (key) => `Pulsó ${key}`,
    scroll: "Desplazó la página",
    back: "Volvió atrás",
  },
  site: {
    wants: (bot) => `${bot} quiere usar el navegador en`,
    asks: (bot, site) => `${bot} pide usar ${site}`,
    why: "Cuando permites un sitio, no vuelve a preguntar por él.",
    allowed: (site) => `Permitiste ${site}`,
    denied: (site) => `Rechazaste ${site}`,
    expired: (site) => `Sin respuesta sobre ${site}`,
  },
};
