// Each bot's own browser (spec 21.8): the panel beside its chat where the
// owner watches it live, its tabs and address, the card where it asks to
// use a new site, and the owner taking the browser into their own hands
// (spec 21.10).

export const browser = {
  heading: "Browser",
  panel: (bot: string) => `${bot}'s browser`,
  show: "Show browser",
  hide: "Hide browser",
  browsing: (bot: string) => `${bot} is using the browser`,
  showInPanel: "Watch in browser",
  close: "Close",
  expand: "Make wider",
  shrink: "Make narrower",
  openOutside: "Open in my browser",
  openFailed: "Could not open the page",
  live: "Live",
  resting: "Resting",
  restingWhy: (bot: string) =>
    `${bot} isn't using the browser, so it rests and costs your computer nothing. It wakes the moment ${bot} or you need it.`,
  loading: "Loading…",
  address: "Address",
  addressHint: "Type an address and press Enter",
  goFailed: "Could not open that address",
  reload: "Reload",
  reloadFailed: "Could not reload the page",
  screen: (bot: string) => `What ${bot} sees`,
  emptyTitle: (bot: string) => `${bot} hasn't opened the browser yet`,
  emptyBody:
    "When it searches or uses a site, you'll see it here live: every page it opens, every click.",
  starting: "Opening the browser…",
  closed: "Browser closed",
  closedBody: "It opens again when the bot needs it. Logins stay.",
  failedTitle: "The browser couldn't open",
  failedBody: (system: string): string =>
    system === "Windows"
      ? "Botloft uses Microsoft Edge, which comes with Windows. Check that it's installed, then ask the bot to try again."
      : "Botloft uses Google Chrome, Chromium or Microsoft Edge. Check that one of them is installed, then ask the bot to try again.",
  details: "Details",
  tabs: {
    label: "Tabs",
    blank: "New tab",
    add: "New tab",
    addFailed: "Could not open a new tab",
    switchFailed: "Could not switch tabs",
    takeFirst: "Take control to switch tabs or open a new one",
  },
  did: {
    open: (site: string) => `Opened ${site}`,
    click: (what: string) => `Clicked ${what}`,
    clickSomewhere: "Clicked",
    type: (what: string) => `Typed in ${what}`,
    typeSomewhere: "Typed",
    select: (what: string) => `Chose in ${what}`,
    press: (key: string) => `Pressed ${key}`,
    scroll: "Scrolled",
    back: "Went back",
  },
  site: {
    wants: (bot: string) => `${bot} wants to use the browser on`,
    asks: (bot: string, site: string) => `${bot} asks to use ${site}`,
    why: "Once you allow a site, it won't ask again for it.",
    allowed: (site: string) => `You allowed ${site}`,
    denied: (site: string) => `You declined ${site}`,
    expired: (site: string) => `No answer about ${site}`,
  },
  hands: {
    take: "Take control",
    /** In the pill under the live screen while the bot has the browser. */
    botHas: (bot: string) => `${bot} is in control`,
    /** Opens why to take the browser, folded under the pill. */
    howItWorks: "How it works",
    takeWhy: (bot: string) =>
      `To sign in to an account or get past a captcha. ${bot} waits meanwhile.`,
    takeFailed: "Could not take the browser",
    holding: "You are in control",
    waits: (bot: string) => `${bot}'s actions in the browser wait until you give it back.`,
    leaves: (bot: string) => `${bot} goes on in the tab you leave open.`,
    giveBack: (bot: string) => `Done, give it back to ${bot}`,
    giveBackFailed: "Could not give the browser back",
    clickToType: "Click the screen to type",
    typing: "What you type goes to the page",
    screen: (bot: string) => `${bot}'s browser, in your hands`,
  },
  /** Teaching the bot a task in its browser (spec 21.13). */
  lesson: {
    teach: "Teach a task",
    teachWhy: (bot: string) =>
      `Do the task here once: ${bot} keeps each step, never what you type.`,
    recording: "Recording the lesson",
    empty: "Do the task on the page: each step shows up here.",
    finish: "Finish the lesson",
    cancel: "Cancel",
    failed: "Could not start or end the lesson",
    title: (bot: string) => `Teach ${bot} a task`,
    name: "Name of the task",
    namePlaceholder: "Check new listings",
    steps: "Steps",
    remove: (n: number) => `Remove step ${n}`,
    makeRoutine: "Make it a routine…",
    sendToBot: (bot: string) => `Send to ${bot} to remember`,
    sendFailed: "Could not send the lesson",
    step: {
      open: (url: string) => `Open ${url}`,
      click: (label: string) => `Click "${label}"`,
      type: (label: string) => `Type in "${label}"`,
      secret: (label: string) => `Type the password in "${label}": ask me to type it`,
      press: (key: string) => `Press ${key}`,
    },
    intro: (name: string) => `How to do "${name}" in the browser, as I showed you:`,
    typedNote: "Where I typed something, I don't show what: use what the task needs, or ask me.",
    forRoutine: "Do it now.",
    forMemory: (name: string) => `Keep this in your memory, so you can do "${name}" when I ask.`,
  },
  window: {
    open: "Sign in in a window",
    why: (bot: string) =>
      `For sites that refuse to sign in here, like Google: ${bot}'s browser opens in a window of its own, and the login stays for ${bot}.`,
    openFailed: "Could not open the window",
    title: "Open in a window",
    body: (bot: string) =>
      `Sign in in the window that opened, then close it. ${bot} waits meanwhile and keeps the login.`,
    hint: "Site won't let you sign in here? Use Sign in in a window, and close the window when you're done.",
  },
  help: {
    needs: (bot: string) => `${bot} needs you in the browser`,
    asks: (bot: string, task: string) => `${bot} asks: ${task}`,
    why: "Take control, do it on the page yourself and give it back. If the site won't let you sign in there, use Sign in in a window in the browser panel. The bot doesn't see what you type in password fields.",
    take: "Take the browser",
    done: "Done",
    wontDo: "I won't do it",
    doneLine: (task: string) => `You did it: ${task}`,
    wontLine: (task: string) => `You didn't do it: ${task}`,
    expired: (task: string) => `No answer: ${task}`,
  },
};
