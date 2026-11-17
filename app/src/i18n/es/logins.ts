import type { logins as en } from "../en/logins";

export const logins: typeof en = {
  title: "Inicios de sesión",
  intro:
    "Inicia sesión una vez en un sitio y elige qué agentes usan ese inicio de sesión. Quien elijas actúa en el sitio como tú.",
  signIn: "Iniciar sesión en un sitio",
  address: "Dirección del sitio",
  placeholder: "reddit.com",
  open: "Abrir ventana",
  inWindow: (site) =>
    `Inicia sesión en ${site} en la ventana que se abrió y ciérrala al terminar. La sesión se guarda cuando la ventana se cierra.`,
  signedIn: (site) => `Iniciaste sesión en ${site}. Elige quién puede usarla.`,
  nothing: (site) =>
    `No se encontró una sesión en ${site}. Inicia sesión antes de cerrar la ventana.`,
  none: "Aún no hay inicios de sesión. Inicia sesión en un sitio para dárselo a tus agentes.",
  list: "Tus inicios de sesión",
  signedAt: (when) => `Sesión iniciada ${when}`,
  agents: (count) => (count === 1 ? "1 agente" : `${count} agentes`),
  noAgent: "Ningún agente aún",
  who: "Quién puede usarlo",
  warning:
    "Quien elijas abre el sitio ya dentro de tu cuenta. Lee con libertad y te pregunta antes de publicar, enviar, comprar o cambiar un ajuste.",
  canUse: (bot) => `${bot} puede usarlo`,
  noAgents: "Aún no hay agentes.",
  again: "Iniciar sesión de nuevo",
  remove: "Quitar inicio de sesión",
  removing: {
    title: (site) => `¿Quitar el inicio de sesión de ${site}?`,
    text: "Cada agente que lo tiene cierra la sesión en ese sitio.",
    confirm: "Quitar",
  },
  failed: "No se pudo cambiar quién usa el inicio de sesión",
  openFailed: "No se pudo abrir la ventana",
  removeFailed: "No se pudo quitar el inicio de sesión",
};
