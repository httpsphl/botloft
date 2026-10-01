import type { Messages } from "../en";

export const account: Messages["account"] = {
  open: (name: string) => `${name}: conta e configurações`,
  plan: (plan: string) => `Plano ${plan}`,
  notSignedIn: "Sem conta do Claude conectada",
  menu: {
    usage: "Uso",
    settings: "Configurações",
    language: "Idioma",
    whatsNew: "Novidades",
    help: "Ajuda",
    openFailed: "Não foi possível abrir a página",
  },
  usage: {
    title: "Uso",
    intro:
      "Quanto do seu plano do Claude os seus bots já usaram. Atualiza sempre que um bot trabalha.",
    empty: "O uso aparece depois da primeira resposta de um bot.",
    windows: {
      five_hour: "Janela de 5 horas",
      seven_day: "Esta semana",
      seven_day_opus: "Esta semana, Opus",
      seven_day_sonnet: "Esta semana, Sonnet",
    },
    used: (percent: number) => `${percent}% usado`,
    resets: (relative: string) => `Renova ${relative}`,
    limited: "Limite atingido. Seus bots esperam até ele renovar.",
    updated: (relative: string) => `Atualizado ${relative}`,
    tokens: {
      title: "Tokens por bot",
      intro:
        "Tokens são os pedaços de texto que um bot lê e escreve. A conta soma o texto novo lido e as respostas escritas. A conversa relida a cada vez aparece à parte, porque pesa bem menos, e a conversa mandada de novo depois de uma pausa também.",
      period: "Período",
      periods: { today: "Hoje", week: "7 dias", month: "30 dias", all: "Tudo" },
      empty: "Nenhum bot trabalhou nesse período.",
      failed: "Não deu para carregar os tokens",
      archived: "arquivado",
      detail: (times: number, reread: string, reloaded: string | null) =>
        `Trabalhou ${times === 1 ? "1 vez" : `${times} vezes`} · ${reread} relidos${
          reloaded === null ? "" : ` · ${reloaded} recarregados`
        }`,
      total: "Todos os bots",
    },
  },
  settings: {
    title: "Configurações",
    background: "Em segundo plano",
    keepWorking: "Continuar trabalhando depois de fechar o Botloft",
    keepWorkingOn: "Seus bots seguem trabalhando e respondendo depois que você fecha esta janela.",
    keepWorkingOff:
      "Ao fechar o Botloft, todos os bots param. Eles continuam de onde pararam quando você abrir de novo.",
    tray: "Mostrar o Botloft perto do relógio",
    trayOn:
      "Com a janela fechada, o ícone fica perto do relógio: um clique abre o Botloft, e ele avisa quando um bot precisa de você.",
    trayOff: "Fechar a janela fecha o Botloft. Os bots seguem trabalhando, sem ícone nem avisos.",
    startWithWindows: "Iniciar com o Windows",
    startWithWindowsOn:
      "Quando você entra no Windows, seus bots voltam a trabalhar sozinhos, sem abrir esta janela.",
    startWithWindowsOff:
      "Depois que você reinicia o computador, os bots esperam você abrir o Botloft.",
    openAtSignIn: "Abrir a janela ao entrar no Windows",
    openAtSignInOn: "A janela do Botloft abre quando você entra no Windows.",
    openAtSignInNearClock: "O Botloft começa perto do relógio, sem abrir a janela.",
    openAtSignInOff: "A janela só abre quando você abrir o Botloft.",
    alerts: "Avisos",
    notifyNeeds: "Quando um bot precisar de você",
    notifyNeedsHint:
      "Um aviso do Windows quando um bot pede permissão ou precisa que você entre na conta, se o Botloft não estiver na frente.",
    notifyDone: "Quando um bot terminar",
    notifyDoneHint: "Um aviso quando um bot termina o que estava fazendo.",
    sound: "Tocar um som",
    soundHint: "Um som curto com cada aviso, também com o Botloft na frente.",
    alertsNeedTray:
      "Com a janela fechada, os avisos só chegam com o Botloft perto do relógio (em Geral).",
    keepAwake: "Não deixar o computador dormir enquanto os bots trabalham",
    keepAwakeHint: "Ele ainda dorme quando você fecha a tampa ou escolhe Suspender.",
    saveFailed: "Não foi possível mudar a configuração",
    pages: "Partes das configurações",
    general: "Geral",
    chat: "Conversa",
    enterSends: "Enter envia a mensagem",
    enterSendsOn: "Shift+Enter começa uma nova linha.",
    enterSendsOff: "Enter começa uma nova linha, e Ctrl+Enter envia.",
    followBot: "Abrir o navegador e as telas quando um bot começar a usar",
    followBotOn: "O painel abre ao lado da conversa, para você ver o que o bot faz.",
    followBotOff: "O botão do painel ganha um ponto, e você abre quando quiser.",
    approvalWait: "Quanto tempo um pedido de permissão espera por você",
    approvalWaitHint: "Sem resposta nesse tempo, o pedido do bot é negado e ele segue sem isso.",
    waitFor: (minutes: number) =>
      minutes < 60 || minutes % 60 !== 0
        ? `${minutes} min`
        : minutes === 60
          ? "1 hora"
          : `${minutes / 60} horas`,
    lessMotion: "Menos animações",
    lessMotionHint:
      "A janela fica parada, como quando os efeitos de animação do Windows estão desligados.",
    appearance: "Aparência",
    theme: "Tema",
    themes: { system: "Sistema", light: "Claro", dark: "Escuro" },
    size: "Tamanho",
    sizeHint: "Ctrl+= e Ctrl+- também mudam o tamanho, e Ctrl+0 volta ao padrão.",
    language: "Idioma",
    archived: "Arquivados",
    archivedIntro:
      "Os bots e as equipes que você arquivou estão parados e fora da vista, e o Botloft ainda guarda as conversas deles. Exclua um para tirá-lo de vez.",
    archivedEmpty: "Nada está arquivado.",
    archivedLoadFailed: "Não foi possível carregar o que está arquivado",
    archivedCrew: (bots: number, when: string) =>
      `Equipe · ${bots === 1 ? "1 bot" : `${bots} bots`} · arquivada ${when}`,
    archivedBot: (crew: string, when: string) => `Bot de ${crew} · arquivado ${when}`,
    deleteArchived: "Excluir",
    deleteArchivedOne: (name: string) => `Excluir ${name}`,
    about: "Sobre",
    claudeCode: (version: string) => `Claude Code ${version}`,
    botloft: (version: string) => `Botloft ${version}`,
    checkUpdates: "Procurar atualizações",
    checking: "Procurando…",
    upToDate: "Você está na versão mais recente.",
    updateFound: (version: string) => `O Botloft ${version} saiu.`,
    seeUpdate: "Ver atualização",
    checkFailed: "Não foi possível procurar atualizações. Tente mais tarde.",
  },
};
