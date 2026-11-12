// Las conversaciones de un agente (spec 40.7).

import type { Messages } from "../en";

export const conversations: Messages["conversations"] = {
  main: "Principal",
  untitled: "Nueva conversación",
  new: "Nueva conversación",
  newHint: (bot: string) => `Empezar otra conversación con ${bot}, junto a esta`,
  newFailed: "No se pudo empezar una nueva conversación",
  current: "Conversación",
  count: (n: number) => `${n} conversaciones`,
  listOf: (bot: string) => `Conversaciones con ${bot}`,
  unread: "respuesta nueva",
  menuOf: (title: string) => `Conversación ${title}`,
  rename: "Cambiar nombre",
  renameTitle: "Cambiar el nombre de la conversación",
  titleLabel: "Nombre",
  save: "Guardar",
  renameFailed: "No se pudo cambiar el nombre de la conversación",
  archive: "Archivar",
  archiveTitle: (title: string) => `¿Archivar ${title}?`,
  archiveBody:
    "Se detiene y sale de la lista. Lo que aún iba hacia ella va a la conversación principal.",
  archiveFailed: "No se pudo archivar la conversación",
};
