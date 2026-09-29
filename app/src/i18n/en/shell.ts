// The window: title bar, language and theme, and the workspace around the
// crews.

export const shell = {
  window: {
    minimize: "Minimize",
    maximize: "Maximize",
    restore: "Restore",
    close: "Close",
  },
  theme: {
    system: "system",
    light: "light",
    dark: "dark",
    label: (theme: string) => `Theme: ${theme}`,
    hint: (theme: string) => `Theme: ${theme} (click to change)`,
  },
  zoom: {
    label: "Size",
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
  pickCrew: "Pick a crew on the left, or create one with the + button.",
  botsCantStart: "Bots can't start",
  signIn: {
    title: "Sign in to Claude",
    body: "Your bots work with your Claude account. Sign in once and they start by themselves.",
  },
};
