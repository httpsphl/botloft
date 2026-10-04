// The bots on the owner's own desktop (spec 24): the request to see an
// app, and what each bot may see in its details.

export const desktop = {
  card: {
    wants: (bot: string) => `${bot} wants to see`,
    asks: (bot: string, app: string) => `${bot} asks to see ${app}`,
    because: (bot: string) => `${bot} says:`,
    means: (bot: string) =>
      `${bot} will read this app's windows and take pictures of them while you are at your computer. Never your passwords. You can take it back in ${bot}'s details.`,
    allowed: (bot: string, app: string) => `${bot} can see ${app}`,
    denied: (bot: string, app: string) => `${bot} may not see ${app}`,
    expired: (app: string) => `No answer about ${app}`,
    wantsUse: (bot: string) => `${bot} wants to use`,
    asksUse: (bot: string, app: string) => `${bot} asks to use ${app}`,
    meansUse: (bot: string) =>
      `${bot} will click, type and choose in this app's windows as you would, without moving your mouse, while you are at your computer. Every action shows in this chat. Never in password fields. You can take it back in ${bot}'s details.`,
    allowedUse: (bot: string, app: string) => `${bot} can use ${app}`,
    deniedUse: (bot: string, app: string) => `${bot} may not use ${app}`,
  },
  panel: {
    label: (bot: string) => `${bot}'s desktop`,
    heading: "Desktop",
    live: "Live",
    stoppedBadge: "Stopped",
    stop: "Stop",
    stopFailed: "Could not stop it",
    resume: "Let it go on",
    resumeFailed: "Could not let it go on",
    stoppedTitle: (bot: string) => `You stopped ${bot} on your desktop`,
    stoppedBody: (bot: string) => `${bot} cannot read or use your apps until you let it go on.`,
    emptyTitle: (bot: string) => `${bot} has not used your desktop yet`,
    emptyBody: "When it reads or uses a window of an app you allowed, the window shows here, live.",
    shortcut: "Ctrl+Alt+End stops every bot on your desktop, even with Botloft closed.",
    waiting: "Waiting for the picture…",
    read: "Read the window",
    click: (target: string) => `Clicked "${target}"`,
    clickSomething: "Clicked a control",
    type: (target: string) => `Typed in "${target}"`,
    typeSomething: "Typed in a field",
    select: (option: string) => `Chose "${option}"`,
    scroll: (target: string) => `Scrolled "${target}"`,
    scrollSomething: "Scrolled",
    showInPanel: "Watch on the desktop panel",
    close: "Close",
  },
  grants: {
    title: "Your desktop",
    none: (bot: string) =>
      `${bot} cannot see any app on your computer. It asks in the chat the first time it needs one.`,
    see: "Can see",
    act: "Can see and use",
    whole: "The whole desktop",
    remove: (app: string) => `Take back ${app}`,
    removeFailed: "Could not take it back",
  },
};
