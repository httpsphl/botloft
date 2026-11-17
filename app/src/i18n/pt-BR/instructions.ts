// Configurações > Instruções para todos (spec 7.10).

import type { instructions as en } from "../en/instructions";

export const instructions: typeof en = {
  page: "Instruções para todos",
  label: "O que todo agente segue",
  intro: "Todo agente lê isto em toda conversa, em qualquer motor. Quanto mais curto, melhor.",
  placeholder:
    "Por exemplo: Me responda em português. Pergunte antes de apagar arquivos. Salve os relatórios na pasta Relatórios.",
  howItWorks: "Como funciona",
  explain:
    "Quando isto discorda de outra coisa, vale o mais específico: o que você pede na conversa, depois as instruções de cada agente, e então estas. Elas vêm antes do que o agente anotou para si e do que outro agente pede a ele, e nunca pulam uma aprovação. Os agentes as leem quando iniciam: um agente disponível começa de novo agora, na mesma conversa, e um que está trabalhando, quando terminar.",
  save: "Salvar",
  saved: "Salvo. Os agentes disponíveis começam de novo para ler.",
  saveFailed: "Não foi possível salvar as instruções",
  loadFailed: "Não foi possível ler as instruções",
};
