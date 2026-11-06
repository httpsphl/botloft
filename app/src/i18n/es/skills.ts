import type { skills as en } from "../en/skills";

export const skills: typeof en = {
  title: "Skills",
  intro:
    "Las skills de tu propio Claude Code, de tu carpeta de skills y de tu cuenta de claude.ai. Elige qué agentes usan cada una: el agente recibe una copia en su carpeta. Solo agentes de Claude Code.",
  none: "No se encontraron skills. Una skill es una carpeta con un SKILL.md en .claude\\skills, en tu carpeta de usuario.",
  failed: "No se pudieron leer tus skills.",
  source: {
    personal: "Tuya",
    claude_ai: "claude.ai",
    kept: "Guardada por Botloft",
  },
  kept: "El original ya no está. Botloft guarda su copia para los agentes que la tienen.",
  agents: (count) =>
    count === 0 ? "Ningún agente aún" : count === 1 ? "1 agente" : `${count} agentes`,
  noDescription: "Sin descripción.",
  warnings: {
    title: "Antes de darla",
    tools: (tools) => `Mientras se usa, ejecuta esto sin preguntarte: ${tools}`,
    commands: "Ejecuta comandos por sí sola cuando se usa.",
    files: "Trae otros archivos, como scripts.",
  },
  who: "Quién puede usarla",
  canUse: (bot) => `${bot} puede usarla`,
  noAgents: "Aún no hay agentes de Claude Code.",
  shareFailed: "No se pudo cambiar quién usa esta skill.",
  request: {
    asks: (bot, name) => `${bot} pide la skill ${name}`,
    wants: (bot, name) => `${bot} quiere usar tu skill ${name}`,
    give: "Dar la skill",
    given: (name, bot) => `Le diste ${name} a ${bot}`,
    declined: (name) => `No diste ${name}`,
    expired: (name) => `La solicitud de ${name} venció`,
  },
};
