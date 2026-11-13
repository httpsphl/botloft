import type { plugins as en } from "../en/plugins";

export const plugins: typeof en = {
  title: "Plugins",
  intro:
    "Los plugins activados en tu propio Claude Code, instalados en él o desde tu cuenta de claude.ai. Un plugin reúne skills, comandos y ayudantes. Elige qué agentes usan cada uno: el agente recibe una copia en su carpeta. Solo agentes de Claude Code.",
  none: "No se encontraron plugins. Los plugins que instalas y activas en Claude Code aparecen aquí.",
  source: {
    claude_code: "Claude Code",
    claude_ai: "claude.ai",
    kept: "Guardado por Botloft",
  },
  kept: "El plugin ya no está en tu Claude Code. Botloft guarda su copia para los agentes que lo tienen.",
  version: (version) => `Versión ${version}`,
  skills: (count) => (count === 0 ? "Ninguna skill" : count === 1 ? "1 skill" : `${count} skills`),
  agents: (count) =>
    count === 0 ? "Ningún agente aún" : count === 1 ? "1 agente" : `${count} agentes`,
  noDescription: "Sin descripción.",
  brings: {
    title: "Qué trae",
    skills: "Skills y comandos",
    agents: "Ayudantes que el agente puede llamar",
    nothing: "Nada que Botloft pueda ver.",
  },
  warnings: {
    title: "Antes de darlo",
    runs: "Ejecuta comandos por sí solo, sin preguntarte, en momentos de la conversación.",
    detail: (list) => `En detalle: ${list}`,
    programs:
      "Trae programas que el agente puede ejecutar en su terminal. Cada uno todavía te pide permiso, como cualquier comando.",
    tools: (names) =>
      `Sus herramientas conectadas no vienen con él: ${names}. Para darle una herramienta a un agente, usa Herramientas.`,
    settings: "Pide ajustes que Botloft no pasa, así que partes de él pueden no funcionar.",
  },
  who: "Quién puede usarlo",
  canUse: (bot) => `${bot} puede usarlo`,
  noAgents: "Aún no hay agentes de Claude Code.",
  shareFailed: "No se pudo cambiar quién usa este plugin.",
  search: "Buscar plugins",
  nothingFound: (query) => `Ningún plugin con "${query}".`,
  everyone: {
    all: "Todos los plugins para todos los agentes",
    allHint:
      "Los agentes de Claude Code usan todos tus plugins, también los que actives después, y los agentes que crees después también.",
    allAsk: (count) =>
      count === 1
        ? "¿Dar el plugin a todos los agentes?"
        : `¿Dar los ${count} plugins a todos los agentes?`,
    allBody:
      "Cada agente de Claude Code recibe una copia de cada uno y vuelve a empezar cuando termina lo que está haciendo, en la misma conversación. Cada agente lee un resumen de las skills de todos los plugins en cada conversación, lo que gasta un poco más de tu plan.",
    allRisky: (count) =>
      count === 1
        ? "1 de ellos ejecuta comandos por sí solo, sin preguntarte."
        : `${count} de ellos ejecutan comandos por sí solos, sin preguntarte.`,
    allGive: "Dar a todos",
    one: "Todos los agentes",
    oneHint: "También los agentes que crees después.",
    byAll: "Todos los plugins están activados para todos, arriba en Plugins.",
    failed: "No se pudo cambiar quién usa tus plugins.",
  },
};
