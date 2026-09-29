import type { Messages } from "../en";

export const common: Messages["common"] = {
  cancel: "Cancelar",
  close: "Fechar",
  resize: "Redimensionar painel",
  dismiss: "Dispensar",
  details: "Detalhes",
  tryAgain: "Tentar de novo",
  errors: {
    closed: "a conexão foi fechada",
    notConnected: "sem conexão com o Botloft",
    connectionLost: "a conexão com o Botloft caiu",
    timedOut: (method: string) => `${method} demorou demais`,
  },
};
