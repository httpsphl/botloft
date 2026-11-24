import type { Messages } from "../en";

export const server: Messages["server"] = {
  connecting: (name) => `Conectando a ${name}…`,
  address: "voce@seu-servidor",
  copy: "Copiar",
  copied: "Copiado",
  sending: (name) => `Enviando ${name} ao servidor…`,
  sendFailed: "Não deu para enviar o arquivo",
  folder: "Pasta no servidor",
  folderHint:
    "Uma pasta do servidor, pelo caminho dela lá. Vazio, uma pasta nova dentro do Botloft. Para código, peça à crew para clonar o repositório na pasta dela.",
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
  own: {
    found: (host) =>
      `Este computador se conecta sozinho a ele, em ${host}. Não precisa abrir nada no terminal.`,
    address: "Endereço do servidor",
    addressHint:
      "Onde este computador encontra o servidor: um IP ou um nome. Troque se você chega a ele de outro jeito.",
    use: "Usar este endereço",
    trouble: {
      unreachable: {
        title: (name) => `Não deu para chegar a ${name}`,
        body: (host) =>
          `O Botloft não conseguiu abrir a conexão com ${host}. Veja se o servidor está ligado e se esse endereço chega até ele. O Botloft continua tentando.`,
      },
      refused: {
        title: (name) => `${name} não aceitou este computador`,
        body: () =>
          "O servidor não aceita mais a chave deste computador; ela pode ter sido removida lá. No servidor, rode sudo botloftd pair e cole a chave nova.",
      },
      host_key: {
        title: (name) => `${name} não é o servidor onde esta chave foi feita`,
        body: (host) =>
          `O computador em ${host} respondeu como outro, então o Botloft não se conectou. Se o servidor foi instalado de novo, rode sudo botloftd pair nele e cole a chave nova.`,
      },
      no_ssh: {
        title: (name) => `Para chegar a ${name}, este computador precisa do OpenSSH`,
        body: () =>
          "O Botloft abre a conexão com o servidor pelo cliente do OpenSSH. No Windows, adicione em Configurações > Sistema > Recursos opcionais > Cliente OpenSSH e tente de novo.",
      },
      failed: {
        title: (name) => `Não deu para conectar a ${name}`,
        body: () =>
          "Algo deu errado ao abrir a conexão; os detalhes dizem o quê. Tente de novo ou cole a chave do servidor outra vez.",
      },
    },
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
    move: "Para levar as crews deste computador: em Configurações > Cópia de segurança, salve uma cópia aqui; depois de conectar, restaure a cópia lá. Então pause as crews daqui, ou elas rodam duas vezes.",
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
  limits: {
    title: "Limites deste servidor",
    hint: "Quanto do servidor o Botloft e os agentes podem usar, para os seus outros sistemas continuarem com a parte deles.",
    memory: "Memória",
    memoryOf: (used, limit, total) => `${used} em uso de ${limit} (o servidor tem ${total})`,
    memoryUsed: (used, total) => `${used} em uso, sem limite (o servidor tem ${total})`,
    memoryOpen: (total) => `Sem limite (o servidor tem ${total})`,
    processors: "Processadores",
    processorsOf: (limit, cpus) => `Até ${limit} de ${cpus}`,
    processorsOpen: (cpus) => `Sem limite (o servidor tem ${cpus})`,
    agents: "Agentes acordados ao mesmo tempo",
    agentsHint: (awake) =>
      `${awake} acordados agora. Os outros esperam dormindo e acordam quando houver vaga.`,
    noLimit: "Sem limite",
    change: "Para mudar a memória e os processadores, rode isto no servidor:",
    container:
      "Aqui o Botloft roda num contêiner: a memória e os processadores são os limites do contêiner, definidos no Docker ou no Coolify.",
    unknown: "Não informado",
    failed: "Não deu para ler os limites",
  },
};
