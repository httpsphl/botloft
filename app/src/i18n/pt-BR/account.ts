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
      title: "Uso por bot",
      intro:
        "Quanto do seu plano semanal cada bot usou, e os tokens por trás disso: os pedaços de texto que um bot lê e escreve.",
      period: "Período",
      periods: {
        hour: "Última hora",
        today: "Hoje",
        week: "7 dias",
        month: "30 dias",
        all: "Tudo",
      },
      empty: "Nenhum bot trabalhou nesse período.",
      failed: "Não deu para carregar os tokens",
      archived: "arquivado",
      /** `crew` is null on the total of all bots. */
      detail: (crew: string | null, times: number) =>
        `${crew === null ? "" : `${crew} · `}Trabalhou ${times === 1 ? "1 vez" : `${times} vezes`}`,
      total: "Todos os bots",
      share: (percent) => `≈ ${percent} da semana`,
      tokensUsed: (count) => `${count} tokens`,
      estimate:
        "≈ Estimativa: o Botloft aprende quanto do seu plano o trabalho dos bots consome pelo quanto o plano semanal sobe enquanto eles trabalham. O que você usa fora do Botloft também faz o plano subir, então os bots podem aparecer com um pouco mais do que usaram.",
      learning:
        "A parte de cada bot no seu plano semanal aparece depois que o plano subir alguns pontos com os bots trabalhando. Até lá, os tokens.",
    },
  },
  backup: {
    exportTitle: "Salvar uma cópia",
    exportIntro:
      "Guarda suas equipes, bots, conversas e a memória de cada bot num arquivo trancado com uma senha, para trazer de volta depois de reinstalar ou em outro computador. Pastas de trabalho que você escolheu fora do Botloft não entram.",
    passphrase: "Senha",
    passphraseHint:
      "Ao menos 8 caracteres. Sem ela a cópia não abre, e o Botloft não a guarda em lugar nenhum.",
    repeat: "Senha de novo",
    mismatch: "As duas senhas não são iguais.",
    export: "Salvar uma cópia…",
    exporting: "Fazendo a cópia…",
    saved: (size) => `Cópia salva (${size}).`,
    notSaved: "A cópia não foi salva.",
    exportFailed: "Não foi possível fazer a cópia",
    importTitle: "Restaurar uma cópia",
    importIntro:
      "Troca tudo o que está neste Botloft por uma cópia. O que existe agora vai para uma pasta à parte: nada é apagado.",
    pick: "Escolher uma cópia…",
    pickTitle: "Escolha uma cópia do Botloft",
    kind: "Cópia do Botloft",
    copyPassphrase: "Senha da cópia",
    open: "Abrir",
    opening: "Abrindo…",
    from: (when) => `Cópia de ${when}`,
    crew: (name, bots) =>
      `${name}: ${bots === 0 ? "nenhum bot" : bots === 1 ? "1 bot" : `${bots} bots`}`,
    workFolders:
      "Estas pastas de trabalho não estão na cópia. Se este é outro computador, ponha os arquivos delas de volta:",
    replaces: "Tudo neste Botloft é trocado. O Botloft reinicia para isso.",
    restore: "Restaurar e reiniciar",
    restoring: "Reiniciando…",
    cancel: "Cancelar",
    openFailed: "Não foi possível abrir a cópia",
    restoreFailed: "Não foi possível restaurar a cópia",
    reasons: {
      short_passphrase: "A senha precisa de ao menos 8 caracteres.",
      wrong_passphrase: "A senha está errada, ou o arquivo está danificado.",
      not_a_backup: "Este arquivo não é uma cópia do Botloft.",
      newer_backup: "Esta cópia é de um Botloft mais novo. Atualize o Botloft antes.",
    } as Record<string, string>,
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
    startWithSystem: (system: string) => `Iniciar com o ${system}`,
    startWithSystemOn: (system: string) =>
      `Quando você entra no ${system}, seus bots voltam a trabalhar sozinhos, sem abrir esta janela.`,
    startWithSystemOff:
      "Depois que você reinicia o computador, os bots esperam você abrir o Botloft.",
    openAtSignIn: (system: string) => `Abrir a janela ao entrar no ${system}`,
    openAtSignInOn: (system: string) => `A janela do Botloft abre quando você entra no ${system}.`,
    openAtSignInNearClock: "O Botloft começa perto do relógio, sem abrir a janela.",
    openAtSignInOff: "A janela só abre quando você abrir o Botloft.",
    alerts: "Avisos",
    notifyNeeds: "Quando um bot precisar de você",
    notifyNeedsHint: (system: string) =>
      `Um aviso do ${system} quando um bot pede permissão ou precisa que você entre na conta, se o Botloft não estiver na frente.`,
    notifyDone: "Quando um bot terminar",
    notifyDoneHint: "Um aviso quando um bot termina o que estava fazendo.",
    markReplies: "Marcar o ícone quando um bot responder",
    markRepliesHint:
      "Um ponto no ícone do Botloft na barra de tarefas enquanto houver conversa que você não leu.",
    sound: "Tocar um som",
    soundHint: "Um som curto com cada aviso, também com o Botloft na frente.",
    appSounds: "Sons do app",
    appSoundsHint:
      "Sons suaves com o Botloft na frente: mensagem enviada, resposta ou arquivo no chat aberto, bot ou equipe nova.",
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
    lessMotionHint: (system: string) =>
      `A janela fica parada, como quando os efeitos de animação do ${system} estão desligados.`,
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
    backup: "Cópia de segurança",
    tools: "Ferramentas conectadas",
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
