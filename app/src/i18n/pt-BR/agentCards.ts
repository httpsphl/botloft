// Cartões que um agente mostra no chat (spec 8.11).

import type { agentCards as en } from "../en/agentCards";

export const agentCards: typeof en = {
  label: (bot, title) => `Cartão de ${bot}: ${title}`,
  updated: (time) => `Atualizado ${time}`,
  send: "Enviar",
  required: "obrigatório",
  pick: "Escolha uma",
  pressed: (label) => `Você escolheu “${label}”`,
  sent: "Você enviou o formulário",
  waiting: (bot) =>
    `${bot} já tem sua resposta. Para perguntar de novo, ele mostra o cartão outra vez.`,
  failed: "Não deu para enviar. Tente de novo.",
  chart: {
    label: (kind, series) => `Gráfico de ${kind} de ${series}`,
    bar: "barras",
    line: "linha",
    asTable: "Ver como tabela",
    asChart: "Ver como gráfico",
    label0: "Rótulo",
  },
  activity: (title) => `Cartão: ${title}`,
};
