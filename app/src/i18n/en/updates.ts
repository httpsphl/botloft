// Updating Botloft from inside the app.

export const updates = {
  available: "Update available",
  title: "Update Botloft",
  ready: (version: string) =>
    `Botloft ${version} is ready. Botloft closes, installs the update and opens again. Your agents pause for a moment and pick up where they left off.`,
  whatsNew: "What's new",
  later: "Later",
  updateNow: "Update now",
  downloading: "Downloading…",
  downloadingPercent: (percent: number) => `Downloading… ${percent}%`,
  installing: "Installing…",
  failedTitle: "The update didn't install",
  failedBody: "Botloft keeps working on the current version.",
};
