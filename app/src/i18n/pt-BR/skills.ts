import type { skills as en } from "../en/skills";

export const skills: typeof en = {
  title: "Skills",
  intro:
    "As skills do seu próprio Claude Code, da sua pasta de skills e da sua conta do claude.ai. Escolha quais agentes usam cada uma: o agente recebe uma cópia na pasta dele. Só agentes do Claude Code.",
  none: "Nenhuma skill encontrada. Uma skill é uma pasta com um SKILL.md em .claude\\skills, na sua pasta de usuário.",
  failed: "Não deu para ler as suas skills.",
  source: {
    personal: "Sua",
    claude_ai: "claude.ai",
    kept: "Guardada pelo Botloft",
  },
  kept: "O original sumiu. O Botloft guarda a cópia dele para os agentes que a têm.",
  agents: (count) =>
    count === 0 ? "Nenhum agente ainda" : count === 1 ? "1 agente" : `${count} agentes`,
  noDescription: "Sem descrição.",
  warnings: {
    title: "Antes de dar",
    tools: (tools) => `Enquanto é usada, ela roda isto sem perguntar a você: ${tools}`,
    commands: "Ela roda comandos sozinha quando é usada.",
    files: "Ela traz outros arquivos, como scripts.",
  },
  who: "Quem pode usar",
  canUse: (bot) => `${bot} pode usar`,
  noAgents: "Nenhum agente do Claude Code ainda.",
  shareFailed: "Não deu para mudar quem usa esta skill.",
  request: {
    asks: (bot, name) => `${bot} pede a skill ${name}`,
    wants: (bot, name) => `${bot} quer usar a sua skill ${name}`,
    give: "Dar a skill",
    given: (name, bot) => `Você deu ${name} a ${bot}`,
    declined: (name) => `Você não deu ${name}`,
    expired: (name) => `O pedido de ${name} expirou`,
  },
  details: {
    label: "Skills",
    none: (bot) => `${bot} não tem nenhuma das suas skills. Ele pede no chat quando uma ajudaria.`,
    choose: "Escolher skills",
  },
};
