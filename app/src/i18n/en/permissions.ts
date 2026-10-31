// Everything the owner let each agent do, in one place (spec 15.1): the
// Permissions page in Settings, and the block in an agent's details.

export const permissions = {
  intro:
    "What each agent may do without asking you again. Take back what you no longer want: the agent asks again the next time.",
  agent: "Agent",
  everyone: "All agents",
  tabs: {
    desktop: "Desktop",
    sites: "Sites",
    rules: "Commands",
    crews: "Other crews",
  },
  empty: {
    desktop:
      "No agent can see apps on your computer. They ask in the chat the first time they need one.",
    sites:
      "No site is allowed for good. Agents ask in the chat the first time they open one in their browser.",
    rules: 'Nothing is allowed for good. When an agent asks, "Always allow" puts it here.',
    crews:
      'Every agent reaches only its own crew. "Always" on a request to reach another one puts it here.',
  },
  sitesModes: 'In "Automatic" and "Without asking" modes, an agent uses any site without asking.',
  allowedOn: (date: string) => `Allowed on ${date}`,
  siteRemove: (site: string) => `Take back ${site}`,
  siteRemoveFailed: "Could not take it back",
  give: "Give an app",
  giveTitle: "Give an agent an app",
  giveApp: "App",
  giveLevel: "It may",
  giveRefresh: "Read the open apps again",
  giveNoApps:
    "No app that can be given is open. Open it on the computer, then read the list again.",
  giveHint: "Only apps open now are listed, never the ones that are never given.",
  giveConfirm: "Give it",
  giveFailed: "Could not give it",
  cancel: "Cancel",
  stopAll: "Stop all now",
  stopAllHint: "Or press Ctrl+Alt+Esc anywhere: every agent stops on your desktop at once.",
  stopAllFailed: "Could not stop them",
  never: "What is never given",
  neverItems: [
    "Botloft itself",
    "Apps running as administrator",
    "The Windows security screens: UAC, Ctrl+Alt+Del and the lock screen",
    "Terminals and command boxes: Command Prompt, PowerShell, Windows Terminal, WSL, Git Bash, the Run box, the Registry Editor and the Task Manager",
    "Password managers and vaults: Credential Manager, Windows Security, 1Password, Bitwarden, KeePass, LastPass and Dashlane",
    "Password fields, in any app",
  ],
  /** The block in an agent's details. */
  section: "Permissions",
  seeAll: "Every agent's permissions",
  sites: "Sites in the browser",
  sitesNone: (bot: string) =>
    `No site yet. ${bot} asks in the chat the first time it opens one in its browser.`,
  /** The link from the desktop panel and the request cards. */
  manage: "Permissions",
  manageHint: "See and take back what each agent may do",
};
