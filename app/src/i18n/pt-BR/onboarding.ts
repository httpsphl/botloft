import type { Messages } from "../en";

export const onboarding: Messages["onboarding"] = {
  tagline: "Equipes de Claude Code sempre ligadas.",
  opening: "Abrindo o Botloft…",
  connecting: "Conectando…",
  restartInBackground: "Reiniciar o Botloft em segundo plano",
  installing: {
    install: "Preparando o Botloft…",
    update: "Atualizando o Botloft…",
    restart: "Reiniciando o Botloft…",
    start: "Iniciando seus bots…",
  },
  installNote: (system: string) =>
    `O Botloft roda seus bots em segundo plano. Em Configurações você escolhe se eles continuam trabalhando depois que você fecha esta janela e se o Botloft inicia com o ${system}.`,
  stopped: {
    title: "O Botloft não conseguiu iniciar",
    body: "O Botloft roda seus bots em segundo plano, e essa parte não iniciou.",
    notRunning: "Não está rodando.",
  },
  outdated: {
    title: "O Botloft não conseguiu terminar a atualização",
    body: "A parte que roda seus bots em segundo plano continua na versão anterior.",
    running: (version: string) => `Versão em execução: ${version}.`,
    stillOld: (version: string) => `Ainda informa a versão ${version}.`,
  },
  late: "Iniciou, mas não respondeu a tempo. O registro fica na pasta logs.",
  foreign: {
    title: "Outro programa está atrapalhando",
    body: (port: number) =>
      `O Botloft precisa da porta ${port} neste computador, e outro programa está usando. Feche esse programa e tente de novo.`,
    detail: (port: number) =>
      `127.0.0.1:${port} responde, mas não é o Botloft. Para usar outra porta, defina "port" no config.toml do Botloft.`,
  },
  mismatch: {
    title: "Este app não combina com o Botloft em execução",
    body: "O Botloft em segundo plano é mais novo que este app. Instale a versão mais recente do Botloft.",
    detail: (version: string, theirs: number, ours: number) =>
      `Versão em execução ${version}, protocolo ${theirs}. Este app fala o protocolo ${ours}.`,
  },
  cantConnect: {
    title: "O Botloft não conseguiu se conectar",
    body: "Tente de novo. Se continuar acontecendo, reinstale o Botloft.",
  },
  welcome: {
    title: "Boas-vindas ao Botloft",
    intro:
      "Uma equipe é um grupo de bots do Claude Code que ficam rodando, trocam mensagens entre si e compartilham uma pasta. Cada equipe começa com um Chefe: diga a ele para que é a equipe, e ele planeja o trabalho e sugere os bots de que precisa.",
    botloft: "Botloft",
    running: (system: string) =>
      `Rodando em segundo plano. Inicia com o ${system}, então seus bots continuam trabalhando depois que você fecha esta janela.`,
    runningNotAtStart:
      "Rodando em segundo plano, então seus bots continuam trabalhando depois que você fecha esta janela.",
    runningWhileOpen: "Rodando enquanto esta janela está aberta. Ao fechá-la, seus bots param.",
    claudeCode: "Claude Code",
    checking: "Verificando…",
    version: (version: string) => `Versão ${version}`,
    account: "Conta do Claude",
    signedIn: "Conectado.",
    signedOut: "Seus bots trabalham com a sua conta do Claude. Entre uma vez e eles ficam prontos.",
    ready: "pronto",
    notReady: "não está pronto",
    stillChecking: "verificando",
    askFirstTitle: "Os bots perguntam antes de mudar as coisas",
    askFirstBody:
      "Quando um bot quer rodar um comando ou editar um arquivo, ele pergunta no chat e espera você permitir ou negar.",
    ideasTitle: "Ou toque em uma ideia",
    createCrew: "Criar sua primeira equipe",
  },
  claudeCode: {
    help: "O Botloft roda seus bots com o Claude Code. Instale ou atualize, abra uma vez para entrar na sua conta, e o Botloft percebe em até 30 segundos.",
    install: "Como instalar o Claude Code",
    openFailed: "Não foi possível abrir o link",
  },
  signIn: {
    button: "Entrar no Claude",
    waiting: "Esperando você entrar…",
    waitingNote:
      "Abriu uma janela com o login do Claude. Termine no navegador; o Botloft continua sozinho.",
    failedTitle: "O login não terminou",
    failedBody: "Tente de novo e termine o login no navegador antes de fechar a janela.",
  },
};
