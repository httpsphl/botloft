// Os arquivos que um bot fez (spec 15.1, 8.5): o painel ao lado do chat.

import type { Messages } from "../en";

export const files: Messages["files"] = {
  heading: "Arquivos",
  panel: (bot) => `Arquivos de ${bot}`,
  show: "Mostrar arquivos",
  showInPanel: "Ver em arquivos",
  hide: "Ocultar arquivos",
  close: "Fechar",
  refresh: "Atualizar",
  fresh: (count) => (count === 1 ? "1 arquivo novo" : `${count} arquivos novos`),
  newTag: "Novo",
  list: "Arquivos",
  emptyTitle: (bot) => `${bot} ainda não fez nenhum arquivo`,
  emptyBody: "O que ele criar para você, como um relatório ou uma planilha, aparece aqui.",
  loadFailed: "Não foi possível carregar os arquivos",
  madeByBot: (bot) => `Feito por ${bot}`,
  back: "Todos os arquivos",
  open: "Abrir",
  reveal: "Mostrar na pasta",
  shared: (bot) => `Arquivos compartilhados por ${bot}`,
  save: "Salvar como…",
  saveOne: (name) => `Salvar uma cópia de ${name}`,
  previewOne: "Ver",
  failed: {
    open: "Não foi possível abrir o arquivo",
    reveal: "Não foi possível mostrar o arquivo",
    save: "Não foi possível salvar o arquivo",
  },
  preview: {
    loading: "Carregando…",
    failed: "Não foi possível mostrar este arquivo",
    none: "Este tipo de arquivo não tem prévia. Abra para ver.",
    tooBig: "Este arquivo é grande demais para mostrar aqui. Abra para ver.",
    cut: "Só o começo do arquivo é mostrado.",
    gone: "Este arquivo não está mais lá.",
  },
};
