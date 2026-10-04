import type { Messages } from "../en";

export const shell: Messages["shell"] = {
  window: {
    minimize: "Minimizar",
    maximize: "Maximizar",
    restore: "Restaurar",
    close: "Fechar",
  },
  /** The button that hides the crews and bots on the left (Ctrl+B). */
  rail: {
    label: "Lugares",
    home: "Início",
  },
  sidebar: {
    hide: "Esconder a lista de bots",
    show: "Mostrar a lista de bots",
  },
  zoom: {
    level: (percent: number, isDefault: boolean) =>
      isDefault ? `${percent}% (padrão)` : `${percent}%`,
  },
  language: {
    label: "Idioma",
    system: (name: string) => `Idioma do sistema: ${name}`,
  },
  connection: {
    lost: "Desconectado",
    reconnecting: "Reconectando…",
  },
  loadFailed: "Não foi possível carregar suas equipes",
  recycleFailed: (path: string) => `A pasta ${path} não foi para a Lixeira e continua lá`,
  botsCantStart: "Os bots não conseguem iniciar",
  signIn: {
    title: "Entre no Claude",
    body: "Seus bots trabalham com a sua conta do Claude. Entre uma vez e eles começam sozinhos.",
  },
};
