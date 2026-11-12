// As conversas de um agente (spec 40.7).

import type { Messages } from "../en";

export const conversations: Messages["conversations"] = {
  main: "Principal",
  untitled: "Nova conversa",
  new: "Nova conversa",
  newHint: (bot: string) => `Começar outra conversa com ${bot}, ao lado desta`,
  newFailed: "Não foi possível começar uma nova conversa",
  current: "Conversa",
  count: (n: number) => `${n} conversas`,
  listOf: (bot: string) => `Conversas com ${bot}`,
  unread: "resposta nova",
  menuOf: (title: string) => `Conversa ${title}`,
  rename: "Renomear",
  renameTitle: "Renomear a conversa",
  titleLabel: "Nome",
  save: "Salvar",
  renameFailed: "Não foi possível renomear a conversa",
  archive: "Arquivar",
  archiveTitle: (title: string) => `Arquivar ${title}?`,
  archiveBody:
    "Ela para e sai da lista. O que ainda estava a caminho dela vai para a conversa principal.",
  archiveFailed: "Não foi possível arquivar a conversa",
};
