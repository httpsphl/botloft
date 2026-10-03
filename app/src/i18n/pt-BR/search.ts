// Busca nas conversas (spec 8.8).

import type { Messages } from "../en";

export const search: Messages["search"] = {
  label: "Buscar",
  open: "Buscar nas conversas (Ctrl+K)",
  field: "Buscar nas conversas",
  placeholder: "Procure palavras nas suas conversas",
  scope: "Onde buscar",
  allCrews: "Todas as equipes",
  hint: "Acha palavras no que você, outros bots e o Botloft escreveram para seus bots, no que eles responderam e nas perguntas deles.",
  tooShort: "Digite pelo menos 2 letras",
  results: "Resultados",
  nothing: (query: string) => `Nada encontrado para “${query}”`,
  more: "Mostrar mais",
  failed: "Não foi possível buscar",
  searching: "Buscando…",
  inCrew: (crew: string) => `em ${crew}`,
  you: "Você",
  botloft: "Botloft",
  goneBot: "Um bot que não está mais na equipe",
  openHere: (bot: string) => `Abrir a conversa com ${bot} neste ponto`,
};
