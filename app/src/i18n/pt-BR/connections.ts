import type { Messages } from "../en";
import { toolAbout, toolKinds } from "./toolAbout";

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
    failed: "Não deu para mudar as ferramentas deste agente",
    reason: "Por quê",
  },
  states: {
    connected: "Conectada",
    pending: "Conectando…",
    needs_auth: "Precisa que você entre",
    failed: "Não conectou",
  },
  claudeAi: {
    looking: "Procurando as ferramentas da sua conta do claude.ai…",
    none: "Nenhuma ferramenta encontrada. Elas aparecem quando o Claude Code usa a sua conta do claude.ai e você conectou alguma lá.",
    refresh: "Procurar de novo",
    toggle: (tool, bot) => `${bot} pode usar ${tool}`,
    manage: "Gerenciar no claude.ai",
    failed: "Não deu para mudar as ferramentas",
    openFailed: "Não deu para abrir o claude.ai",
  },
  page: {
    yours: "Suas",
    discover: "Descobrir",
    search: "Buscar ferramentas",
    add: "Adicionar",
    addClaudeAi: "Do claude.ai",
    addPaste: "Colar uma configuração",
    fromClaudeAi: "claude.ai",
    yourOwn: "Sua",
    connected: "Conectada",
    agents: (count) => (count === 1 ? "1 agente" : `${count} agentes`),
    noAgent: "Nenhum agente ainda",
    checked: (time) => `Sua conta do claude.ai, conferida às ${time}`,
    discoverIntro:
      "Conecte no claude.ai uma vez, com o seu login. Depois volte e procure de novo: elas aparecem aqui para os seus agentes.",
    connect: (tool) => `Conectar ${tool} no claude.ai`,
    more: "Mais de 8 mil ferramentas no claude.ai",
    nothingFound: (query) => `Nenhuma ferramenta com “${query}”.`,
    noneYet: "Nenhuma ferramenta ainda. Descubra o que dá para conectar, ou cole uma configuração.",
    who: "Quem pode usar",
    onlyClaude: "Só agentes do Claude Code usam as ferramentas do claude.ai.",
    noAgents: "Nenhum agente ainda.",
    canUse: (bot) => `${bot} pode usar`,
    manage: "Gerenciar no claude.ai",
    close: "Fechar",
  },
  about: toolAbout,
  toolKinds,
  request: {
    asks: (bot, tool) => `${bot} pede ${tool}`,
    wants: (bot, tool) => `${bot} quer usar o seu ${tool}`,
    how: (bot) =>
      `Da sua conta do claude.ai, sem login no navegador. ${bot} reinicia com ele e continua.`,
    give: "Permitir",
    given: (tool, bot) => `Você deu ${tool} a ${bot}`,
    declined: (tool) => `Você não deu ${tool}`,
    expired: (tool) => `${tool} não foi respondido a tempo`,
  },
};
