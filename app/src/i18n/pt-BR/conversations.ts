// As conversas de um agente (spec 40.7).

import type { Messages } from "../en";

export const conversations: Messages["conversations"] = {
  main: "Principal",
  untitled: "Nova conversa",
  new: "Nova conversa",
  newHint: (bot: string) => `Começar outra conversa com ${bot}, ao lado desta`,
  newFailed: "Não foi possível começar uma nova conversa",
  current: "Conversa",
};
