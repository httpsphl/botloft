// O navegador de cada bot (spec 21.8): o painel ao lado do chat onde o dono
// assiste ao vivo, as abas e o endereço, o cartão em que o bot pede para
// usar um site novo, e o dono assumindo o navegador com as próprias mãos
// (spec 21.10).

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
  resting: "Em descanso",
  restingWhy: (bot) =>
    `${bot} não está usando o navegador, então ele descansa e não gasta o seu computador. Volta na hora em que ${bot} ou você precisar.`,
  loading: "Carregando…",
  address: "Endereço",
  addressHint: "Digite um endereço e aperte Enter",
  goFailed: "Não foi possível abrir esse endereço",
  reload: "Recarregar",
  reloadFailed: "Não foi possível recarregar a página",
  screen: (bot) => `O que ${bot} vê`,
  emptyTitle: (bot) => `${bot} ainda não abriu o navegador`,
  emptyBody:
    "Quando ele pesquisar ou usar um site, você vê aqui ao vivo: cada página que ele abre, cada clique.",
  starting: "Abrindo o navegador…",
  closed: "Navegador fechado",
  closedBody: "Ele abre de novo quando o bot precisar. Os logins continuam.",
  failedTitle: "O navegador não abriu",
  failedBody: (system: string) =>
    system === "Windows"
      ? "O Botloft usa o Microsoft Edge, que vem com o Windows. Confira se ele está instalado e peça ao bot para tentar de novo."
      : "O Botloft usa o Google Chrome, o Chromium ou o Microsoft Edge. Confira se um deles está instalado e peça ao bot para tentar de novo.",
  details: "Detalhes",
  tabs: {
    label: "Abas",
    blank: "Nova aba",
    add: "Nova aba",
    addFailed: "Não foi possível abrir uma nova aba",
    switchFailed: "Não foi possível trocar de aba",
    takeFirst: "Assuma o controle para trocar de aba ou abrir uma nova",
  },
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
  hands: {
    take: "Assumir o controle",
    takeWhy: (bot) =>
      `Para entrar numa conta ou passar de um captcha. ${bot} espera enquanto isso.`,
    takeFailed: "Não foi possível assumir o navegador",
    holding: "Você está no controle",
    waits: (bot) => `As ações de ${bot} no navegador esperam você devolver.`,
    leaves: (bot) => `${bot} continua na aba que você deixar aberta.`,
    giveBack: (bot) => `Pronto, devolver para ${bot}`,
    giveBackFailed: "Não foi possível devolver o navegador",
    clickToType: "Clique na tela para digitar",
    typing: "O que você digita vai para a página",
    screen: (bot) => `Navegador de ${bot}, nas suas mãos`,
  },
  window: {
    open: "Entrar numa janela",
    why: (bot) =>
      `Para sites que recusam o login aqui, como o Google: o navegador de ${bot} abre numa janela própria, e o login fica com ${bot}.`,
    openFailed: "Não foi possível abrir a janela",
    title: "Aberto numa janela",
    body: (bot) =>
      `Entre na conta na janela que abriu e depois feche-a. ${bot} espera enquanto isso e fica com o login.`,
    hint: "O site não deixa entrar aqui? Use Entrar numa janela e feche a janela quando terminar.",
  },
  help: {
    needs: (bot) => `${bot} precisa de você no navegador`,
    asks: (bot, task) => `${bot} pede: ${task}`,
    why: "Assuma o controle, faça isso na página e devolva. Se o site não deixar entrar por ali, use Entrar numa janela no painel do navegador. O bot não vê o que você digita em campos de senha.",
    take: "Assumir o navegador",
    done: "Pronto",
    wontDo: "Não vou fazer",
    doneLine: (task) => `Você fez: ${task}`,
    wontLine: (task) => `Você não fez: ${task}`,
    expired: (task) => `Sem resposta: ${task}`,
  },
};
