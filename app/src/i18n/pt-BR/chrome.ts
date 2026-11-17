import type { chrome as en } from "../en/chrome";

export const chrome: typeof en = {
  tab: "Seu Chrome",
  short: "Chrome",
  intro:
    "Agentes que você escolher podem usar o seu próprio Chrome, logado nas suas contas, pela extensão Claude in Chrome. Só agentes do Claude Code.",
  none: "Nenhum agente tem o seu Chrome. Um agente pede no chat quando uma tarefa precisa dele.",
  givenOn: (when) => `Dado ${when}`,
  give: "Dar o seu Chrome a um agente",
  take: (bot) => `Tirar o seu Chrome de ${bot}`,
  takeFailed: "Não deu para tirar o seu Chrome.",
  giveFailed: "Não deu para dar o seu Chrome.",
  sites: 'Os sites que você permitiu sempre ficam em Comandos, como "No seu Chrome".',
  using: (bot) => `${bot} está usando o seu Chrome agora.`,
  nobody: "Nenhum agente está usando o seu Chrome agora.",
  stop: "Parar",
  stopFailed: "Não deu para parar o agente.",
  oneAtATime:
    "Um agente por vez usa o seu Chrome, cada um nas próprias abas; você continua usando como sempre. Parar recusa os próximos passos do agente ali até o turno dele acabar.",
  dialog: {
    title: "Dar o seu Chrome a um agente",
    agent: "Agente",
    risks: (bot) => [
      `O seu Chrome está logado nas suas contas: ali, ${bot} age como você.`,
      `${bot} pode errar e mandar, publicar, comprar ou apagar algo como se fosse você.`,
      "Um texto num site pode tentar enganar o agente para fazer outra coisa.",
      "Cada site novo pergunta a você antes, e você vê cada passo no chat.",
    ],
    understood: "Entendi os riscos",
    confirm: "Dar o meu Chrome",
    cancel: "Cancelar",
    noAgent: "Todos os agentes do Claude Code já têm o seu Chrome.",
  },
  request: {
    asks: (bot) => `${bot} pede o seu Chrome`,
    wants: (bot) => `${bot} quer usar o seu Chrome`,
    give: "Dar o meu Chrome",
    given: (bot) => `Você deu o seu Chrome a ${bot}`,
    declined: "Você não deu o seu Chrome",
    expired: "O pedido do seu Chrome expirou",
  },
  details: {
    label: "Seu Chrome",
  },
};
