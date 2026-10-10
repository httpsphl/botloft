import type { Messages } from "../en";

export const updates: Messages["updates"] = {
  available: "Atualização disponível",
  title: "Atualizar o Botloft",
  ready: (version: string) =>
    `O Botloft ${version} está pronto. O Botloft fecha, instala a atualização e abre de novo. Seus agentes pausam por um instante e continuam de onde pararam.`,
  whatsNew: "Novidades",
  later: "Depois",
  updateNow: "Atualizar agora",
  downloading: "Baixando…",
  downloadingPercent: (percent: number) => `Baixando… ${percent}%`,
  installing: "Instalando…",
  failedTitle: "A atualização não foi instalada",
  failedBody: "O Botloft continua funcionando na versão atual.",
};
