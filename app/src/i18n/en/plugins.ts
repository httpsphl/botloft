// The owner's Claude Code plugins in the agents (spec 41): Settings >
// Plugins, and a plugin's dialog with what it brings, what to know before
// giving it and who may use it.

export const plugins = {
  title: "Plugins",
  intro:
    "The plugins turned on in your own Claude Code, installed there or from your claude.ai account. A plugin brings skills, commands and helpers together. Choose which agents use each one: an agent gets a copy in its folder. Only agents on Claude Code can.",
  none: "No plugins found. The plugins you install and turn on in Claude Code show up here.",
  source: {
    claude_code: "Claude Code",
    claude_ai: "claude.ai",
    kept: "Kept by Botloft",
  },
  kept: "The plugin is no longer in your Claude Code. Botloft keeps its copy for the agents that have it.",
  version: (version: string) => `Version ${version}`,
  skills: (count: number) =>
    count === 0 ? "No skills" : count === 1 ? "1 skill" : `${count} skills`,
  agents: (count: number) =>
    count === 0 ? "No agent yet" : count === 1 ? "1 agent" : `${count} agents`,
  noDescription: "No description.",
  brings: {
    title: "What it brings",
    skills: "Skills and commands",
    agents: "Helpers the agent can call",
    nothing: "Nothing Botloft can see.",
  },
  warnings: {
    title: "Before giving it",
    runs: "It runs commands by itself, without asking you, at moments of the conversation.",
    detail: (list: string) => `In detail: ${list}`,
    programs:
      "It brings programs the agent can run in its terminal. Each one still asks you, like any other command.",
    tools: (names: string) =>
      `Its connected tools do not come along: ${names}. To give an agent a tool, use Tools.`,
    settings: "It asks for settings Botloft does not pass on, so parts of it may not work.",
  },
  who: "Who may use it",
  canUse: (bot: string) => `${bot} can use it`,
  noAgents: "No agents on Claude Code yet.",
  shareFailed: "Could not change who uses this plugin.",
  search: "Search plugins",
  nothingFound: (query: string) => `No plugin matches "${query}".`,
  everyone: {
    all: "Every plugin for every agent",
    allHint:
      "The agents on Claude Code use all your plugins, the ones you turn on later too, and so do the agents you make later.",
    allAsk: (count: number) =>
      count === 1 ? "Give the plugin to every agent?" : `Give all ${count} plugins to every agent?`,
    allBody:
      "Each agent on Claude Code gets a copy of each one and starts again when it finishes what it is doing, in the same conversation. Each agent reads a summary of the skills of every plugin in every conversation, which uses a little more of your plan.",
    allRisky: (count: number) =>
      count === 1
        ? "1 of them runs commands by itself, without asking you."
        : `${count} of them run commands by themselves, without asking you.`,
    allGive: "Give to every agent",
    one: "Every agent",
    oneHint: "The agents you make later too.",
    byAll: "Every plugin is on for every agent, at the top of Plugins.",
    failed: "Could not change who uses your plugins.",
  },
};
