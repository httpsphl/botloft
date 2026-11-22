import type { Messages } from "../en";

export const server: Messages["server"] = {
  connecting: (name) => `Conectando a ${name}…`,
  address: "voce@seu-servidor",
  copy: "Copiar",
  copied: "Copiado",
  useThisComputer: "Usar este computador",
  unreachable: {
    title: (name) => `Não achei ${name}`,
    body: "Este app chega ao servidor por um túnel que começa neste computador. Abra o túnel num terminal e deixe aberto:",
    retrying: "O Botloft continua tentando e conecta assim que o túnel abrir.",
  },
  refused: {
    title: (name) => `${name} não aceitou este computador`,
    body: "A chave dele pode ter sido removida lá. No servidor, rode sudo botloftd pair e cole a chave nova, ou volte para este computador.",
  },
  form: {
    title: "Conectar a um servidor",
    intro:
      "Rode o Botloft num servidor seu e os agentes continuam trabalhando com este computador desligado. No servidor, rode sudo botloftd pair e cole a chave que ele mostrar.",
    key: "Chave de conexão",
    show: "Mostrar a chave",
    hide: "Esconder a chave",
    placeholder: "botloft1.…",
    invalid: "Isso não é uma chave do Botloft. Copie a linha inteira que começa com botloft1.",
    found: (name) => `Servidor: ${name}`,
    tunnel: "Este computador chega a ele por um túnel. Abra o túnel num terminal e deixe aberto:",
    connect: "Conectar",
  },
  where: {
    title: "Onde os agentes rodam",
    thisComputer: "Neste computador",
    thisComputerHint: "O Botloft roda aqui, em segundo plano.",
    onServer: (name) => `No servidor ${name}`,
    onServerHint:
      "Este app mostra os agentes que rodam no servidor. Eles continuam trabalhando com este computador desligado.",
    toServer: "Conectar a um servidor",
    toThisComputer: "Usar este computador",
    toThisComputerTitle: "Usar o Botloft deste computador?",
    toThisComputerText:
      "Os agentes do servidor continuam rodando. Este app passa a mostrar os deste computador, e dá para conectar ao servidor de novo com a mesma chave.",
  },
  devices: {
    title: "Aparelhos",
    hint: "Computadores que chegam a estes agentes com uma chave própria.",
    none: "Nenhum outro aparelho tem chave.",
    thisOne: "Este computador",
    seen: (time) => `Conectou por último ${time}`,
    never: "Nunca conectou",
    remove: "Remover",
    removeTitle: (name) => `Remover ${name}?`,
    removeText: "A chave dele para de valer na hora, e ele é desconectado.",
    failed: "Não deu para remover",
  },
};
