import type { logins as en } from "../en/logins";

export const logins: typeof en = {
  title: "Logins",
  intro:
    "Entre uma vez num site e escolha quais agentes usam esse login. Quem você escolher age no site como você.",
  signIn: "Entrar num site",
  address: "Endereço do site",
  placeholder: "reddit.com",
  open: "Abrir janela",
  inWindow: (site) =>
    `Entre em ${site} na janela que abriu e feche-a quando terminar. O login fica salvo quando a janela fecha.`,
  signedIn: (site) => `Você entrou em ${site}. Escolha quem pode usar.`,
  nothing: (site) => `Nenhum login foi achado em ${site}. Entre na conta antes de fechar a janela.`,
  none: "Nenhum login ainda. Entre num site para dar o login aos seus agentes.",
  list: "Seus logins",
  signedAt: (when) => `Entrou ${when}`,
  agents: (count) => (count === 1 ? "1 agente" : `${count} agentes`),
  noAgent: "Nenhum agente ainda",
  who: "Quem pode usar",
  warning:
    "Quem você escolher abre o site já dentro da sua conta. Lê à vontade e pergunta antes de publicar, mandar, comprar ou mudar uma configuração.",
  canUse: (bot) => `${bot} pode usar`,
  noAgents: "Nenhum agente ainda.",
  again: "Entrar de novo",
  remove: "Remover login",
  removing: {
    title: (site) => `Remover o login de ${site}?`,
    text: "Todo agente que o tem sai da conta nesse site.",
    confirm: "Remover",
  },
  failed: "Não deu para mudar quem usa o login",
  openFailed: "Não deu para abrir a janela",
  removeFailed: "Não deu para remover o login",
};
