// O navegador de cada bot (spec 21.8): o painel ao lado do chat onde o dono
// assiste ao vivo, e o cartão em que o bot pede para usar um site novo.

import type { Messages } from "../en";

export const browser: Messages["browser"] = {
  heading: "Navegador",
  panel: (bot) => `Navegador de ${bot}`,
  show: "Mostrar navegador",
  hide: "Ocultar navegador",
  browsing: (bot) => `${bot} está usando o navegador`,
  showInPanel: "Ver no navegador",
  close: "Fechar",
  expand: "Alargar",
  shrink: "Estreitar",
  openOutside: "Abrir no meu navegador",
  openFailed: "Não foi possível abrir a página",
  live: "Ao vivo",
  loading: "Carregando…",
  address: "Endereço",
  screen: (bot) => `O que ${bot} vê`,
  emptyTitle: (bot) => `${bot} ainda não abriu o navegador`,
  emptyBody:
    "Quando ele pesquisar ou usar um site, você vê aqui ao vivo: cada página que ele abre, cada clique.",
  starting: "Abrindo o navegador…",
  closed: "Navegador fechado",
  closedBody: "Ele abre de novo quando o bot precisar. Os logins continuam.",
  failedTitle: "O navegador não abriu",
  failedBody:
    "O Botloft usa o Microsoft Edge, que vem com o Windows. Confira se ele está instalado e peça ao bot para tentar de novo.",
  details: "Detalhes",
  tabs: (count) => (count === 1 ? "1 aba" : `${count} abas`),
  did: {
    open: (site) => `Abriu ${site}`,
    click: (what) => `Clicou em ${what}`,
    clickSomewhere: "Clicou",
    type: (what) => `Escreveu em ${what}`,
    typeSomewhere: "Escreveu",
    select: (what) => `Escolheu em ${what}`,
    press: (key) => `Apertou ${key}`,
    scroll: "Rolou a página",
    back: "Voltou",
  },
  site: {
    wants: (bot) => `${bot} quer usar o navegador em`,
    asks: (bot, site) => `${bot} pede para usar ${site}`,
    why: "Depois que você permite um site, ele não pergunta de novo.",
    allowed: (site) => `Você permitiu ${site}`,
    denied: (site) => `Você recusou ${site}`,
    expired: (site) => `Sem resposta sobre ${site}`,
  },
};
