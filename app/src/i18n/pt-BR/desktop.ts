import type { Messages } from "../en";

export const desktop: Messages["desktop"] = {
  card: {
    wants: (bot) => `${bot} quer ver`,
    asks: (bot, app) => `${bot} pede para ver ${app}`,
    because: (bot) => `${bot} diz:`,
    means: (bot) =>
      `${bot} vai ler as janelas desse app e tirar fotos delas enquanto você estiver no computador. Nunca as suas senhas. Você pode tirar isso nos detalhes de ${bot}.`,
    allowed: (bot, app) => `${bot} pode ver ${app}`,
    denied: (bot, app) => `${bot} não pode ver ${app}`,
    expired: (app) => `Sem resposta sobre ${app}`,
    wantsUse: (bot) => `${bot} quer usar`,
    asksUse: (bot, app) => `${bot} pede para usar ${app}`,
    meansUse: (bot) =>
      `${bot} vai clicar, escrever e escolher nas janelas desse app como você faria, sem mexer no seu mouse, enquanto você estiver no computador. Cada ação aparece neste chat. Nunca em campos de senha. Você pode tirar isso nos detalhes de ${bot}.`,
    allowedUse: (bot, app) => `${bot} pode usar ${app}`,
    deniedUse: (bot, app) => `${bot} não pode usar ${app}`,
  },
  panel: {
    label: (bot) => `Desktop de ${bot}`,
    heading: "Desktop",
    live: "Ao vivo",
    stoppedBadge: "Parado",
    stop: "Parar",
    stopFailed: "Não foi possível parar",
    resume: "Deixar continuar",
    resumeFailed: "Não foi possível deixar continuar",
    stoppedTitle: (bot) => `Você parou ${bot} no seu desktop`,
    stoppedBody: (bot) => `${bot} não lê nem usa seus apps até você deixar continuar.`,
    emptyTitle: (bot) => `${bot} ainda não usou seu desktop`,
    emptyBody:
      "Quando ele ler ou usar uma janela de um app que você liberou, a janela aparece aqui, ao vivo.",
    shortcut: "Ctrl+Alt+End para todos os bots no seu desktop, mesmo com o Botloft fechado.",
    waiting: "Esperando a imagem…",
    read: "Leu a janela",
    click: (target) => `Clicou em "${target}"`,
    clickSomething: "Clicou num controle",
    type: (target) => `Escreveu em "${target}"`,
    typeSomething: "Escreveu num campo",
    select: (option) => `Escolheu "${option}"`,
    scroll: (target) => `Rolou "${target}"`,
    scrollSomething: "Rolou",
    showInPanel: "Ver no painel do desktop",
    close: "Fechar",
  },
  grants: {
    title: "Seu desktop",
    none: (bot) =>
      `${bot} não vê nenhum app do seu computador. Ele pede no chat na primeira vez que precisar de um.`,
    see: "Pode ver",
    act: "Pode ver e usar",
    whole: "O desktop inteiro",
    remove: (app) => `Tirar ${app}`,
    removeFailed: "Não foi possível tirar",
  },
};
