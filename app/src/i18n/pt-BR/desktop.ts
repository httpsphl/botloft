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
