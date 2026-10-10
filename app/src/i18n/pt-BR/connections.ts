import type { Messages } from "../en";

export const connections: Messages["connections"] = {
  title: "Ferramentas conectadas",
  intro:
    "Ferramentas suas que os agentes podem usar além das que o Botloft já dá, como um leitor de LinkedIn ou um sistema da sua empresa. Cada agente recebe só as que você liga para ele, e ainda pergunta antes de usá-las.",
  loadFailed: "Não deu para ler as ferramentas conectadas",
  none: "Nenhuma ferramenta conectada ainda.",
  add: "Conectar uma ferramenta…",
  usedBy: (names) => `Usada por ${names}`,
  unused: "Nenhum agente a usa ainda",
  kinds: {
    stdio: "Um programa neste computador",
    http: "Uma ferramenta em um endereço",
  },
  edit: (name) => `Editar ${name}`,
  remove: (name) => `Remover ${name}`,
  adding: {
    title: "Conectar uma ferramenta",
    paste: "Cole as configurações dela",
    pasteHint:
      "O texto do arquivo .mcp.json dela, ou as configurações que vieram com a ferramenta.",
    name: "Nome",
    nameHint: "Só é preciso quando as configurações coladas vêm sem nome.",
    purpose: "Para que serve? (opcional)",
    purposeHint: "Os agentes leem isto para saber quando usá-la.",
    found: (count) => (count === 1 ? "Achei 1 ferramenta." : `Achei ${count} ferramentas.`),
    program: (name) =>
      `${name} é um programa. Ele vai rodar no seu computador com as suas permissões, fora das pastas que o Botloft guarda para os seus agentes. Conecte só programas em que você confia.`,
    address: (name) =>
      `${name} é uma ferramenta em um endereço. O que o seu agente mandar para ela sai deste computador.`,
    command: "O que ela executa",
    understood: "Entendi e confio nela",
    connect: "Conectar",
    connecting: "Conectando…",
    failed: "Não deu para conectar a ferramenta",
  },
  problems: {
    empty: "Cole primeiro as configurações da ferramenta.",
    notJson: "Isso não é um JSON válido. Cole o texto exatamente como está no arquivo.",
    noServers: "Não há nenhuma ferramenta neste texto.",
    needsName: "Estas configurações não têm nome: escreva um abaixo.",
    unsupported: (server) => `${server} usa um tipo de conexão que o Botloft ainda não aceita.`,
    unknownField: (server, field) =>
      `${server} tem uma configuração que o Botloft não conhece: "${field}". Tire-a e tente de novo.`,
    badValue: (server, field) =>
      `${server} tem algo errado em "${field}". Confira e tente de novo.`,
  },
  editing: {
    title: (name) => `Editar ${name}`,
    note: "Para mudar o que ela executa, o endereço ou as senhas, remova a ferramenta e conecte de novo.",
    save: "Salvar",
    failed: "Não deu para salvar a ferramenta",
  },
  removing: {
    title: (name) => `Remover ${name}?`,
    text: "Os agentes que a usam a perdem e começam de novo quando estiverem livres. O que você permitiu para sempre com ela é esquecido.",
    confirm: "Remover",
    failed: "Não deu para remover a ferramenta",
  },
  bot: {
    title: "Ferramentas conectadas",
    none: "Nenhuma ferramenta conectada ainda. Você pode conectá-las em Configurações, em Ferramentas conectadas.",
    toggle: (tool, bot) => `${bot} pode usar ${tool}`,
    failed: "Não deu para mudar as ferramentas deste agente",
    reason: "Por quê",
  },
  states: {
    connected: "Conectada",
    pending: "Conectando…",
    needs_auth: "Precisa que você entre",
    failed: "Não conectou",
  },
};
