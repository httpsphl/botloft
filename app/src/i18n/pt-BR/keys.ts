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
  scan: {
    title: "Chaves coladas nas conversas",
    intro:
      "O Botloft procura no que você e seus agentes escreveram textos com cara de chave: tokens do GitHub, Slack, Stripe, OpenAI, Anthropic, Google, AWS e tokens de login. Mostra só o começo e o fim de cada uma.",
    run: "Procurar chaves",
    again: "Procurar de novo",
    none: "Nenhuma chave encontrada nas conversas.",
    found: (count: number) => (count === 1 ? "1 chave encontrada" : `${count} chaves encontradas`),
    places: (count: number) => (count === 1 ? "em 1 lugar" : `em ${count} lugares`),
    keep: "Guardar como chave",
    keepTitle: "Guardar como chave de",
    agent: "Agente",
    goneAgent: "um agente que não existe mais",
    remove: "Apagar do Botloft",
    removeTitle: (preview: string) => `Apagar ${preview} do Botloft?`,
    removeBody:
      "Todo lugar onde ela aparece vai ficar [chave removida]. Não tem volta, e a chave continua valendo no serviço de onde veio.",
    done: "Pronto",
    failed: "Não foi possível procurar chaves",
    scrubFailed: "Não foi possível mudar as conversas",
    reachTitle: "Troque a chave no serviço de onde ela veio",
    reach:
      "Ela já passou por lugares que o Botloft não controla. O Botloft não mexe no histórico de conversa do próprio motor (começar uma conversa nova com o agente deixa ele para trás) nem nas cópias na nuvem já feitas (a próxima sai sem a chave; as antigas se apagam em Conta).",
  },
  pasted: (names: string[], bot: string) =>
    `${names.length === 1 ? `A chave ${names[0]} já está guardada` : `As chaves ${names.join(", ")} já estão guardadas`}, então ${bot} recebeu o nome no lugar do valor.`,
};
