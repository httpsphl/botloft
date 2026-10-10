import type { Messages } from "../en";

export const messages: Messages["messages"] = {
  calls: {
    label: "Agentes chamando uns aos outros",
    caller: (bot) => `${bot}:`,
    calling: (bot) => `Chamando ${bot}`,
    picked: (bot) => `${bot} atendeu`,
  },
  composer: {
    messageTo: "Mensagem para",
    recipient: "Destinatário",
    placeholder: (handle: string | undefined) =>
      `Escreva para @${handle ?? "agente"}. Ctrl+Enter envia.`,
    send: "Enviar",
  },
  delivery: {
    delivered: "Entregue",
    read: "Lida",
    readTitle: "O agente começou a trabalhar nela",
    delivering: "Entregando",
    waiting: "Aguardando o agente",
    retrying: (when: string, attempts: number) =>
      `Nova tentativa ${when}, após ${attempts} ${attempts === 1 ? "tentativa sem sucesso" : "tentativas sem sucesso"}`,
    notDelivered: "Não entregue",
    retry: "Tentar de novo",
    retryFailed: "Não foi possível tentar entregar de novo",
  },
  failed: {
    button: (count: number) => `${count} não ${count === 1 ? "entregue" : "entregues"}`,
    title: "Mensagens não entregues",
    retryAll: "Tentar todas de novo",
    explanation:
      "O Botloft desistiu após várias tentativas. Tentar de novo coloca a mensagem de volta na fila; ela é enviada quando o agente estiver pronto.",
    to: (name: string) => `Para ${name}`,
    tries: (count: number) => `${count} ${count === 1 ? "tentativa" : "tentativas"}`,
  },
  row: {
    you: "Você",
    system: "Botloft",
    goneBot: "um agente que não está mais na equipe",
    to: "para",
    task: "Tarefa",
    result: "Resultado",
    due: (when: string) => `prazo ${when}`,
    taskStatus: {
      open: "aberta",
      done: "concluída",
      failed: "falhou",
      cancelled: "cancelada",
      expired: "expirada",
    },
  },
  timeline: {
    list: "Mensagens",
    loadFailed: "Não foi possível carregar as mensagens",
    loadOlder: "Carregar mensagens anteriores",
    loading: "Carregando mensagens…",
  },
};
