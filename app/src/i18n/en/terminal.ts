// The bot's computer (spec 15.1): its terminal, with the commands it ran,
// and the dock that moves between its browser, terminal and files.

export const terminal = {
  heading: "Terminal",
  panel: (bot: string) => `${bot}'s terminal`,
  /** The title of the terminal window. */
  title: (bot: string) => `${bot} — commands`,
  close: "Close",
  showInPanel: "Show in terminal",
  running: "running…",
  loading: "Loading the commands…",
  empty: (bot: string) => `${bot} has not run any command yet. What it runs shows up here, live.`,
  dock: {
    label: "Computer",
    browser: "Browser",
    terminal: "Terminal",
    files: "Files",
    desktop: "Desktop",
  },
};
