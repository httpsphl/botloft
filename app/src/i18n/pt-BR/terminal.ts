import type { Messages } from "../en";

export const terminal: Messages["terminal"] = {
  heading: "Terminal",
  panel: (bot) => `Terminal de ${bot}`,
  title: (bot) => `${bot} — comandos`,
  close: "Fechar",
  showInPanel: "Ver no terminal",
  running: "rodando…",
  loading: "Carregando os comandos…",
  empty: (bot) => `${bot} ainda não rodou nenhum comando. O que ele rodar aparece aqui, ao vivo.`,
  dock: {
    label: "Computador",
    browser: "Navegador",
    terminal: "Terminal",
    files: "Arquivos",
  },
};
