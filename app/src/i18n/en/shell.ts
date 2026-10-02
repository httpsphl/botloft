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
  sidebar: {
    hide: "Hide the bots list",
    show: "Show the bots list",
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
  botsCantStart: "Bots can't start",
  signIn: {
    title: "Sign in to Claude",
    body: "Your bots work with your Claude account. Sign in once and they start by themselves.",
  },
};
