// The overview beside an agent's chat (spec 33).

export const overview = {
  heading: "Overview",
  panel: (bot: string) => `Overview of ${bot}`,
  show: "Show overview",
  hide: "Hide overview",
  close: "Close",
  context: {
    title: "Conversation space",
    used: (percent: string) => `${percent} used`,
    of: (used: string, total: string) => `${used} of ${total}`,
    bar: (percent: string) => `Conversation space, ${percent} used`,
    unknown: (bot: string) => `${bot} hasn't said how full its conversation is yet.`,
    noParts: "This engine only gives the total, not what fills it.",
    parts: "What fills it",
    free: "Free",
    kind: {
      messages: "Conversation",
      instructions: "Claude Code's instructions",
      tools: "Claude Code's tools",
      connected_tools: "Connected tools",
      memory: "Memory",
      skills: "Skills",
      agents: "Subagents",
      buffer: "Room kept to compact",
      other: "Other",
    },
    memory: "Memory files",
    memoryHint: (bot: string) => `What ${bot} reads at the start of every conversation.`,
  },
};
