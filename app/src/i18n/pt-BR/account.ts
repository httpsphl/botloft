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
  },
  settings: {
    title: "Configurações",
    background: "Em segundo plano",
    keepWorking: "Continuar trabalhando depois de fechar o Botloft",
    keepWorkingOn: "Seus bots seguem trabalhando e respondendo depois que você fecha esta janela.",
    keepWorkingOff:
      "Ao fechar o Botloft, todos os bots param. Eles continuam de onde pararam quando você abrir de novo.",
    startWithWindows: "Iniciar com o Windows",
    startWithWindowsOn:
      "Quando você entra no Windows, seus bots voltam a trabalhar sozinhos, sem abrir esta janela.",
    startWithWindowsOff:
      "Depois que você reinicia o computador, os bots esperam você abrir o Botloft.",
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
