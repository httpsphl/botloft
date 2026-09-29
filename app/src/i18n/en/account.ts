// The account area at the bottom of the sidebar: who the owner is, the
// plan's usage, settings, language, what's new and help.

export const account = {
  open: (name: string) => `${name}: account and settings`,
  plan: (plan: string) => `${plan} plan`,
  notSignedIn: "Not signed in to Claude",
  menu: {
    usage: "Usage",
    settings: "Settings",
    language: "Language",
    whatsNew: "What's new",
    help: "Help",
    openFailed: "Could not open the page",
  },
  usage: {
    title: "Usage",
    intro: "How much of your Claude plan your bots have used. It updates whenever a bot works.",
    empty: "Usage shows up after a bot's first reply.",
    windows: {
      five_hour: "5-hour window",
      seven_day: "This week",
      seven_day_opus: "This week, Opus",
      seven_day_sonnet: "This week, Sonnet",
    } as Record<string, string>,
    used: (percent: number) => `${percent}% used`,
    resets: (relative: string) => `Resets ${relative}`,
    limited: "Limit reached. Your bots wait until it resets.",
    updated: (relative: string) => `Updated ${relative}`,
  },
  settings: {
    title: "Settings",
    appearance: "Appearance",
    theme: "Theme",
    themes: { system: "System", light: "Light", dark: "Dark" },
    size: "Size",
    sizeHint: "Ctrl+= and Ctrl+- change the size too, and Ctrl+0 goes back to the default.",
    language: "Language",
    about: "About",
    claudeCode: (version: string) => `Claude Code ${version}`,
    botloft: (version: string) => `Botloft ${version}`,
  },
};
