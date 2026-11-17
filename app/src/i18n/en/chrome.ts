// The owner's own Chrome in the agents (spec 37): its tab on the
// Permissions page, the risks before giving it, the agent's request in the
// chat and who is using it now.

export const chrome = {
  tab: "Your Chrome",
  /** The tab of the Permissions page, short so five fit side by side. */
  short: "Chrome",
  intro:
    "Agents you choose can use your own Chrome, signed in to your accounts, through the Claude in Chrome extension. Only agents on Claude Code can.",
  none: "No agent has your Chrome. An agent asks in its chat when a task needs it.",
  givenOn: (when: string) => `Given ${when}`,
  give: "Give your Chrome to an agent",
  take: (bot: string) => `Take your Chrome back from ${bot}`,
  takeFailed: "Could not take your Chrome back.",
  giveFailed: "Could not give your Chrome.",
  sites: 'Sites you allowed for good are under Commands, as "In your Chrome".',
  using: (bot: string) => `${bot} is using your Chrome now.`,
  nobody: "No agent is using your Chrome now.",
  stop: "Stop",
  stopFailed: "Could not stop the agent.",
  oneAtATime:
    "One agent at a time uses your Chrome, each in its own tabs; you keep using it as always. Stop refuses the agent's next steps there until its turn ends.",
  dialog: {
    title: "Give your Chrome to an agent",
    agent: "Agent",
    risks: (bot: string) => [
      `Your Chrome is signed in to your accounts: there, ${bot} acts as you.`,
      `${bot} can make mistakes and send, post, buy or delete something as if it were you.`,
      "A text on a site can try to trick the agent into doing something else.",
      "Each new site asks you first, and you see every step in the chat.",
    ],
    understood: "I understand the risks",
    confirm: "Give my Chrome",
    cancel: "Cancel",
    noAgent: "Every agent on Claude Code already has your Chrome.",
  },
  request: {
    asks: (bot: string) => `${bot} asks for your Chrome`,
    wants: (bot: string) => `${bot} wants to use your Chrome`,
    give: "Give my Chrome",
    given: (bot: string) => `You gave your Chrome to ${bot}`,
    declined: "You did not give your Chrome",
    expired: "The request for your Chrome expired",
  },
  details: {
    label: "Your Chrome",
  },
};
