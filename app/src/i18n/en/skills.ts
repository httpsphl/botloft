// The owner's skills in the agents (spec 38): Settings > Skills, a skill's
// dialog with who may use it, the agent's request in the chat and the
// agent's details.

export const skills = {
  title: "Skills",
  intro:
    "The skills of your own Claude Code, from your skills folder and your claude.ai account. Choose which agents use each one: an agent gets a copy in its folder. Only agents on Claude Code can.",
  none: "No skills found. A skill is a folder with a SKILL.md in .claude\\skills, in your user folder.",
  failed: "Could not read your skills.",
  source: {
    personal: "Yours",
    claude_ai: "claude.ai",
    kept: "Kept by Botloft",
  },
  kept: "The original is gone. Botloft keeps its copy for the agents that have it.",
  agents: (count: number) =>
    count === 0 ? "No agent yet" : count === 1 ? "1 agent" : `${count} agents`,
  noDescription: "No description.",
  warnings: {
    title: "Before you give it",
    tools: (tools: string) => `While it is used, it runs these without asking you: ${tools}`,
    commands: "It runs commands by itself when it is used.",
    files: "It brings other files, such as scripts.",
  },
  who: "Who can use",
  canUse: (bot: string) => `${bot} can use it`,
  noAgents: "No agent on Claude Code yet.",
  shareFailed: "Could not change who uses this skill.",
  request: {
    asks: (bot: string, name: string) => `${bot} asks for the skill ${name}`,
    wants: (bot: string, name: string) => `${bot} wants to use your skill ${name}`,
    give: "Give the skill",
    given: (name: string, bot: string) => `You gave ${name} to ${bot}`,
    declined: (name: string) => `You did not give ${name}`,
    expired: (name: string) => `The request for ${name} expired`,
  },
};
