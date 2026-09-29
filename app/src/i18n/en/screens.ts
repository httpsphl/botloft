// The design area (spec 22.5): the HTML screens a bot makes, live beside
// its chat while it writes them.

export const screens = {
  heading: "Screens",
  panel: (bot: string) => `${bot}'s screens`,
  show: "Show screens",
  hide: "Hide screens",
  drawing: (bot: string) => `${bot} is drawing a screen`,
  showInPanel: "Show in screens",
  close: "Close",
  expand: "Make wider",
  shrink: "Make narrower",
  zoomIn: "Zoom in",
  zoomOut: "Zoom out",
  fit: "Fit",
  board: "Screens",
  open: (name: string) => `Open ${name}`,
  writing: "Writing…",
  back: "All screens",
  openFile: "Open",
  reveal: "Show in folder",
  device: "Device",
  devices: {
    desktop: "Computer",
    tablet: "Tablet",
    mobile: "Phone",
  },
  more: (count: number) => `Show ${count} more`,
  emptyTitle: (bot: string) => `${bot} hasn't made any screens yet`,
  emptyBody:
    "Every HTML page it writes shows up here, and you watch it take shape while it writes.",
  loadFailed: "Couldn't load the screens",
  gone: "This screen is no longer there.",
  failed: {
    open: "Could not open the screen",
    reveal: "Could not show the file",
  },
};
