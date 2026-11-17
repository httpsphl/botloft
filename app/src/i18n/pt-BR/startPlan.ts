// O plano que um agente mostra antes de um trabalho longo (spec 10.5).

import type { Messages } from "../en";

export const startPlan: Messages["chat"]["startPlan"] = {
  ready: (bot: string) => `${bot} vai começar assim`,
  alternative: "Ou um jeito menor",
  startsIn: (seconds: number) => `Começa sozinho em ${seconds} s`,
  starting: "Começando…",
  startNow: "Começar agora",
  onlyThis: "Fazer só isto",
  change: "Mudar",
  sendChange: "Enviar",
  noteLabel: (bot: string) => `O que ${bot} deve fazer em vez disso`,
  notePlaceholder: "O que deve mudar?",
  cancel: "Cancelar",
  failed: "Não foi possível responder ao plano",
  started: "Você mandou começar",
  startedAlone: "Começou sozinho",
  didAlternative: "Fazendo só o jeito menor",
  changed: "Você pediu mudanças",
  cancelled: "Você cancelou",
};
