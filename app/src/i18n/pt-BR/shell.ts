import type { Messages } from "../en";

export const shell: Messages["shell"] = {
  window: {
    minimize: "Minimizar",
    maximize: "Maximizar",
    restore: "Restaurar",
    close: "Fechar",
  },
  zoom: {
    level: (percent: number, isDefault: boolean) =>
      isDefault ? `${percent}% (padrão)` : `${percent}%`,
  },
  language: {
    label: "Idioma",
    system: (name: string) => `Idioma do sistema: ${name}`,
  },
  connection: {
    lost: "Desconectado",
    reconnecting: "Reconectando…",
  },
  loadFailed: "Não foi possível carregar suas equipes",
  pickCrew: "Escolha uma equipe à esquerda, ou crie uma com o botão +.",
  botsCantStart: "Os bots não conseguem iniciar",
  signIn: {
    title: "Entre no Claude",
    body: "Seus bots trabalham com a sua conta do Claude. Entre uma vez e eles começam sozinhos.",
  },
};
