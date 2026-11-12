// Las conversaciones de un agente (spec 40.7).

import type { Messages } from "../en";

export const conversations: Messages["conversations"] = {
  main: "Principal",
  untitled: "Nueva conversación",
  new: "Nueva conversación",
  newHint: (bot: string) => `Empezar otra conversación con ${bot}, junto a esta`,
  newFailed: "No se pudo empezar una nueva conversación",
  current: "Conversación",
};
