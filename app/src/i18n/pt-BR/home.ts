import type { Messages } from "../en";

export const home: Messages["home"] = {
  label: "Início",
  title: "Início",
  summary: (waiting: number, working: number) =>
    [
      waiting > 0 && `${waiting} ${waiting === 1 ? "coisa esperando" : "coisas esperando"} você`,
      working > 0 && `${working} trabalhando`,
    ]
      .filter(Boolean)
      .join(" · ") || "Tudo calmo",
  needsYou: "Precisa de você",
  working: "Trabalhando agora",
  replies: "Respostas novas",
  crews: "Crews",
  today: {
    title: "Hoje",
    nothing: "Nada feito ainda hoje.",
    tasks: "Tarefas feitas",
    failed: (count: number) => `${count} ${count === 1 ? "falhou" : "falharam"}`,
    requests: "Pedidos respondidos",
    files: "Arquivos editados",
    messages: "Mensagens entre agentes",
    busiest: "Mais trabalhou",
  },
  empty: {
    title: "Nada esperando você",
    body: "Quando um agente pedir sua permissão, uma decisão ou uma chave, aparece aqui, de todas as crews.",
  },
  request: {
    seeAll: "Ver o pedido",
    openChat: (bot: string) => `Abrir a conversa com ${bot}`,
    openInChat: "Abrir no chat",
    cut: "Longo demais para mostrar inteiro aqui. Para permitir, abra no chat.",
    atComputer: "Este muda as coisas de vez, então você responde no chat.",
    gone: "Já foi respondido",
  },
  key: {
    title: "Precisa de uma chave",
    inChat: "Você cola a chave no chat, num cartão que a guarda fora da conversa.",
  },
  failed: {
    line: (count: number) =>
      count === 1 ? "1 mensagem não foi entregue" : `${count} mensagens não foram entregues`,
    show: "Ver e tentar de novo",
  },
  signedOut: {
    line: (bot: string) => `${bot} parou: o motor dele não está conectado à conta`,
    open: "Abrir",
  },
};
