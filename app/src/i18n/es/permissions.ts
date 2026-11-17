import type { Messages } from "../en";

export const permissions: Messages["permissions"] = {
  intro:
    "Lo que cada agente puede hacer sin volver a preguntarte. Quita lo que ya no quieras: el agente lo pide de nuevo la próxima vez.",
  agent: "Agente",
  everyone: "Todos los agentes",
  tabs: {
    desktop: "Escritorio",
    sites: "Sitios",
    rules: "Comandos",
    crews: "Otros equipos",
  },
  empty: {
    desktop:
      "Ningún agente puede ver apps de tu computadora. Lo piden en el chat la primera vez que necesitan una.",
    sites:
      "Ningún sitio permitido para siempre. Los agentes lo piden en el chat la primera vez que abren uno en su navegador.",
    rules:
      'Nada permitido para siempre. Cuando un agente pide algo, "Permitir siempre" lo pone aquí.',
    crews:
      'Cada agente solo alcanza su propio equipo. "Siempre" en un pedido para alcanzar otro lo pone aquí.',
  },
  sitesModes:
    'En los modos "Automático" y "Sin preguntar", el agente usa cualquier sitio sin pedirlo.',
  allowedOn: (date) => `Permitido el ${date}`,
  siteRemove: (site) => `Quitar ${site}`,
  siteRemoveFailed: "No se pudo quitar",
  give: "Dar una app",
  giveTitle: "Dar una app a un agente",
  giveApp: "App",
  giveLevel: "Puede",
  giveRefresh: "Leer de nuevo las apps abiertas",
  giveNoApps:
    "No hay abierta ninguna app que se pueda dar. Ábrela en la computadora y lee la lista de nuevo.",
  giveHint: "La lista muestra solo las apps abiertas ahora, nunca las que nunca se dan.",
  giveConfirm: "Dar",
  giveFailed: "No se pudo dar",
  cancel: "Cancelar",
  stopAll: "Detener a todos ahora",
  stopAllHint:
    "O presiona Ctrl+Alt+Esc en cualquier lugar: todos los agentes se detienen en tu escritorio al momento.",
  stopAllFailed: "No se pudo detenerlos",
  never: "Lo que nunca se da",
  neverItems: [
    "El propio Botloft",
    "Apps que corren como administrador",
    "Las pantallas de seguridad de Windows: UAC, Ctrl+Alt+Supr y la pantalla de bloqueo",
    "Terminales y cajas de comandos: Símbolo del sistema, PowerShell, Windows Terminal, WSL, Git Bash, la caja Ejecutar, el Editor del Registro y el Administrador de tareas",
    "Gestores de contraseñas y bóvedas: Administrador de credenciales, Seguridad de Windows, 1Password, Bitwarden, KeePass, LastPass y Dashlane",
    "Campos de contraseña, en cualquier app",
  ],
  section: "Permisos",
  seeAll: "Permisos de todos los agentes",
  sites: "Sitios en el navegador",
  sitesNone: (bot) =>
    `Ningún sitio todavía. ${bot} lo pide en el chat la primera vez que abre uno en su navegador.`,
  manage: "Permisos",
  manageHint: "Ver y quitar lo que cada agente puede hacer",
};
