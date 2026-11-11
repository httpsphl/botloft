// The history of an agent's memory and instructions (spec 8.10): each
// version with who made it, its changes, and a way back to it.

export const versions = {
  /** The buttons that open it. */
  memoryHistory: "Memory history",
  instructionsHistory: "Instructions history",
  memory: "Memory",
  memoryAbout: (bot: string) =>
    `What ${bot} keeps from one conversation to the next, in the CLAUDE.md of its folder. ${bot} writes it; every change is kept.`,
  title: {
    memory: (bot: string) => `${bot}'s memory history`,
    instructions: (bot: string) => `${bot}'s instructions history`,
  },
  intro: {
    memory: (bot: string) =>
      `Every change to ${bot}'s memory, newest first. Botloft keeps this history outside ${bot}'s folder, so ${bot} cannot change it.`,
    instructions: (bot: string) => `Every change to ${bot}'s instructions, newest first.`,
  },
  empty: {
    memory: (bot: string) =>
      `No changes yet. The history starts the first time ${bot} or someone else changes its memory.`,
    instructions: "No changes yet. The history starts the first time the instructions change.",
  },
  list: "Versions",
  author: {
    start: "How it was at first",
    owner: "You changed it",
    restored: (at: string) => `You went back to the version of ${at}`,
    agent: (bot: string) => `${bot} changed it`,
    maker: "The Maker changed it",
    outside: "Changed outside Botloft",
  },
  now: "Now",
  emptyText: "(empty)",
  showChanges: "Show the changes",
  restore: "Go back to this version",
  restoreConfirm: {
    title: "Go back to this version?",
    memory: (bot: string, at: string) =>
      `${bot}'s memory goes back to how it was on ${at}. The version of now stays in the history, so you can undo this too. ${bot} reads it the next time it starts a conversation.`,
    instructions: (bot: string, at: string) =>
      `${bot}'s instructions go back to how they were on ${at}. The version of now stays in the history, so you can undo this too.`,
    confirm: "Go back",
  },
  failed: {
    load: "Could not load the history",
    restore: "Could not go back to that version",
  },
};
