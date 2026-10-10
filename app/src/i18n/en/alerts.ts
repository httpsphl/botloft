// The icon near the clock and the Windows notifications (spec 15.1).

export const alerts = {
  tray: {
    tooltip: (status: string) => `Botloft: ${status}`,
    working: (count: number) => (count === 1 ? "1 agent working" : `${count} agents working`),
    needsYou: (bot: string) => `${bot} needs you`,
    waiting: "Something is waiting for you",
    idle: "No agent working",
    open: "Open Botloft",
    pause: "Pause every crew",
    resume: "Resume every crew",
    quit: "Quit Botloft",
  },
  approval: (bot: string) => `${bot} asks for your permission`,
  signIn: (bot: string) => `${bot} needs you to sign in to Claude`,
  signInBody: "Open Botloft to sign in.",
  done: (bot: string) => `${bot} finished`,
  crew: (crew: string) => `Crew ${crew}`,
};
