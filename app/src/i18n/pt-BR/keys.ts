// Chaves dos agentes (spec 32).

import type { Messages } from "../en";

export const keys: Messages["keys"] = {
  card: {
    asks: (bot: string) => `${bot} precisa de uma chave`,
    why: "Para quê",
    saved: (name: string) => `Você guardou a chave ${name}`,
    dismissed: (name: string) => `Você não deu a chave ${name}`,
    notNow: "Agora não",
    notNowHint: (bot: string) => `Fecha o pedido e avisa ${bot} que a chave não vem agora.`,
    dismissFailed: "Não foi possível fechar o pedido",
  },
  form: {
    name: "Nome",
    nameHint: "Letras maiúsculas, números e _, como GITHUB_TOKEN. O agente lê a chave por ele.",
    value: "Chave",
    valueHint: "Fica só neste computador, fora da conversa. Ninguém consegue ler de volta aqui.",
    description: "Descrição (opcional)",
    descriptionPlaceholder: "Para que serve",
    show: "Mostrar a chave",
    hide: "Esconder a chave",
    save: "Salvar",
    cancel: "Cancelar",
    saveFailed: "Não foi possível salvar a chave",
    restarts: (bot: string) => `${bot} reinicia para recebê-la, sem perder a conversa.`,
  },
  details: {
    title: "Chaves",
    none: (bot: string) =>
      `Nenhuma chave ainda. Quando ${bot} precisar de um token ou senha de outro serviço, guarde aqui em vez de colar no chat.`,
    howItWorks: "Como funciona",
    explain: (bot: string) =>
      `${bot} recebe cada chave como uma variável com o nome dela, e o Botloft troca pelo nome uma chave que aparece no chat. Não é uma garantia: se ${bot} imprimir a chave mesmo assim, ela fica no histórico do próprio motor. A melhor proteção é uma chave que pode pouco.`,
    add: "Adicionar chave",
    replace: (name: string) => `Trocar ${name}`,
    remove: (name: string) => `Apagar ${name}`,
    missing: "Falta o valor",
    missingHint:
      "O nome veio com uma cópia de segurança, o valor não. Troque para dar a chave de novo.",
    changed: (when: string) => `mudou ${when}`,
    removeTitle: (name: string) => `Apagar a chave ${name}?`,
    removeBody: (bot: string) =>
      `${bot} reinicia sem ela. A chave continua valendo no serviço de onde veio: para cancelar lá, revogue nesse serviço.`,
    removeConfirm: "Apagar",
    removeFailed: "Não foi possível apagar a chave",
  },
  pasted: (names: string[], bot: string) =>
    `${names.length === 1 ? `A chave ${names[0]} já está guardada` : `As chaves ${names.join(", ")} já estão guardadas`}, então ${bot} recebeu o nome no lugar do valor.`,
};
