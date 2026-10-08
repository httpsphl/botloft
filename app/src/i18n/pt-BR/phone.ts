// A página do celular (spec 28.7).

import type { Messages } from "../en";

export const phone: Messages["phone"] = {
  pair: {
    title: "Conectar este celular",
    intro:
      "Este celular vai aprovar os pedidos dos seus bots e responder às perguntas deles, de qualquer lugar. As conversas e os arquivos ficam no seu computador.",
    nameLabel: "Nome deste celular",
    connect: "Conectar",
    connecting: "Conectando…",
    compare: (code: string) =>
      `Confira se o computador mostra ${code} também. Depois aperte Conectar no computador.`,
    waiting: "Esperando você aceitar no computador…",
    notALink:
      "Para conectar um celular, aponte a câmera dele para o código que aparece no computador: Configurações, Cópia de segurança, Celular.",
    expired: "O código venceu ou já foi usado. Peça um novo no computador.",
    offline: "Não foi possível falar com o Botloft. Confira a conexão e tente de novo.",
    failed: "Não foi possível conectar. Peça um novo código no computador.",
    names: { iphone: "iPhone", ipad: "iPad", android: "Celular Android", other: "Celular" },
  },
  inbox: {
    title: "Esperando você",
    empty: "Nada esperando você",
    emptyBody: "Quando um bot precisar de você, aparece aqui.",
    loading: "Vendo o que está esperando…",
    computerOff: "O computador está desligado ou sem internet. Os pedidos continuam esperando lá.",
    noConnection: "Sem conexão. Tentando de novo…",
    thisPhone: "Este celular",
    inCrew: (crew: string) => `em ${crew}`,
  },
  approval: {
    asks: (bot: string) => `${bot} pede para`,
    explanationBy: (bot: string) => `${bot} escreveu isto`,
    noExplanation: (bot: string) => `${bot} não disse para que serve. Vale negar e perguntar.`,
    showAll: "Mostrar o pedido inteiro",
    hideAll: "Esconder",
    allow: "Permitir",
    deny: "Negar",
    sending: "Enviando…",
    noteLabel: "Recado para o bot (opcional)",
    cut: "Longo demais para ler no celular. Aqui você só pode negar, ou responder no computador.",
    atComputer: "Este pedido precisa de você no computador. Aqui você só pode negar.",
    failed: "Não foi enviado. Confira a conexão e tente de novo.",
    ended: { allowed: "Permitido", denied: "Negado", expired: "Venceu" },
  },
  question: {
    asks: (bot: string) => `${bot} pergunta`,
    pick: "Escolha uma resposta",
    answerLabel: "Sua resposta",
    placeholder: "Escreva sua resposta",
    send: "Responder",
    sending: "Enviando…",
    dismiss: "Descartar",
    dismissHint: (bot: string) => `Fecha a pergunta sem avisar ${bot}. Para ele saber, responda.`,
    failed: "Não foi enviado. Confira a conexão e tente de novo.",
    ended: { answered: "Você respondeu", dismissed: "Descartada" },
  },
  settings: {
    title: "Este celular",
    back: "Voltar",
    name: "Nome",
    disconnect: "Desconectar este celular",
    disconnectTitle: "Desconectar este celular?",
    disconnectText:
      "Ele deixa de receber pedidos. Para usar de novo, conecte outra vez no computador.",
    cancel: "Cancelar",
    install:
      "Para receber avisos, ponha o Botloft na tela inicial: toque em Compartilhar e depois em Adicionar à Tela de Início.",
  },
  cut: {
    revokedTitle: "Este celular foi desconectado",
    leftTitle: "Celular desconectado",
    body: "Para usar de novo, conecte pelo Botloft no computador: Configurações, Cópia de segurança, Celular.",
    ok: "OK",
  },
};
