// Búsqueda en los chats (spec 8.8).

import type { Messages } from "../en";

export const search: Messages["search"] = {
  label: "Buscar",
  open: "Buscar en los chats (Ctrl+K)",
  field: "Buscar en los chats",
  placeholder: "Busca palabras en tus chats",
  scope: "Dónde buscar",
  allCrews: "Todos los equipos",
  hint: "Encuentra palabras en lo que tú, otros agentes y Botloft escribieron a tus agentes, en lo que respondieron y en sus preguntas.",
  tooShort: "Escribe al menos 2 letras",
  results: "Resultados",
  nothing: (query: string) => `No se encontró nada para “${query}”`,
  more: "Mostrar más",
  failed: "No se pudo buscar",
  searching: "Buscando…",
  inCrew: (crew: string) => `en ${crew}`,
  you: "Tú",
  botloft: "Botloft",
  goneBot: "Un agente que ya no está en el equipo",
  openHere: (bot: string) => `Abrir el chat con ${bot} en este punto`,
};
