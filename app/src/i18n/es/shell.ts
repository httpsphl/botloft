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
    hide: "Ocultar la lista de agentes",
    show: "Mostrar la lista de agentes",
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
  botsCantStart: "Los agentes no pueden iniciar",
  signIn: {
    title: "Inicia sesión en Claude",
    body: "Tus agentes trabajan con tu cuenta de Claude. Inicia sesión una vez y empiezan solos.",
    endedElsewhere:
      "Los agentes se detuvieron: terminó su inicio de sesión de Claude. Vuelve a iniciarla en la computadora donde corre Botloft, y vuelven solos.",
    inTerminal: "En la terminal de esa computadora, ejecuta:",
    endsTitle: "El inicio de sesión de Claude de los agentes termina pronto",
    endsBody: (from, by) =>
      `Termina entre ${from} y ${by}. Vuelve a iniciarla antes, o los agentes se detienen hasta que alguien lo haga.`,
    endsNow:
      "Puede terminar en cualquier momento. Vuelve a iniciarla, o los agentes se detienen hasta que alguien lo haga.",
  },
};
