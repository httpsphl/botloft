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
