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
    hide: "Esconder a lista de agentes",
    show: "Mostrar a lista de agentes",
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
  botsCantStart: "Os agentes não conseguem iniciar",
  signIn: {
    title: "Entre no Claude",
    body: "Seus agentes trabalham com a sua conta do Claude. Entre uma vez e eles começam sozinhos.",
    endedElsewhere:
      "Os agentes pararam: o login do Claude deles acabou. Entre de novo no computador onde o Botloft roda, e eles voltam sozinhos.",
    inTerminal: "No terminal daquele computador, rode:",
    endsTitle: "O login do Claude dos agentes acaba logo",
    endsBody: (from, by) =>
      `Ele acaba entre ${from} e ${by}. Entre de novo antes disso, ou os agentes param até alguém entrar.`,
    endsNow:
      "Ele pode acabar a qualquer momento. Entre de novo, ou os agentes param até alguém entrar.",
  },
};
