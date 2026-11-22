// The window: title bar, language and theme, and the workspace around the
// crews.

export const shell = {
  window: {
    minimize: "Minimize",
    maximize: "Maximize",
    restore: "Restore",
    close: "Close",
  },
  /** The button that hides the crews and bots on the left (Ctrl+B). */
  /** The icons at the far left. */
  rail: {
    label: "Places",
    home: "Home",
  },
  sidebar: {
    hide: "Hide the agents list",
    show: "Show the agents list",
  },
  zoom: {
    level: (percent: number, isDefault: boolean) =>
      isDefault ? `${percent}% (default)` : `${percent}%`,
  },
  language: {
    label: "Language",
    /** The "follow Windows" entry, with the language that gives. */
    system: (name: string) => `System language: ${name}`,
  },
  connection: {
    lost: "Disconnected",
    reconnecting: "Reconnecting…",
  },
  loadFailed: "Could not load your crews",
  /** A deleted bot's or crew's folder that could not go to the Recycle Bin. */
  recycleFailed: (path: string) =>
    `The folder ${path} did not go to the Recycle Bin and is still there`,
  botsCantStart: "Agents can't start",
  signIn: {
    title: "Sign in to Claude",
    body: "Your agents work with your Claude account. Sign in once and they start by themselves.",
    endedElsewhere:
      "Your agents stopped: Claude's sign-in for them ended. Sign in again where Botloft runs, and they start by themselves.",
    inTerminal: "In that computer's terminal, run:",
    endsTitle: "Claude's sign-in for your agents ends soon",
    endsBody: (from: string, by: string) =>
      `It ends between ${from} and ${by}. Sign in again before that, or the agents stop until someone does.`,
    endsNow: "It may end any day now. Sign in again, or the agents stop until someone does.",
  },
};
