// The island at the top of the screen (spec 31): the pill and the panel
// with what waits for the owner. What the agents ask is data and never
// passes through here.

export const island = {
  /** The whole island, for screen readers. */
  label: "Botloft at the top of the screen",
  waiting: (count: number) => (count === 1 ? "1 waiting" : `${count} waiting`),
  working: (count: number) => (count === 1 ? "1 working" : `${count} working`),
  /** Which card of how many: "2 of 3". */
  position: (at: number, of: number) => `${at} of ${of}`,
  previous: "Previous",
  next: "Next",
  tabs: {
    label: "Island tabs",
    home: "Home",
  },
  openApp: "Open Botloft",
  hide: "Hide",
  card: {
    allow: "Allow",
    deny: "Deny",
    openInApp: "Open in Botloft",
    /** A request too long to show whole: it can only be denied here. */
    cut: "Too long to show whole here. Open it in Botloft to read it all.",
    atComputer: "Needs you in Botloft",
    asks: (bot: string) => `${bot} asks`,
    answerLabel: (bot: string) => `Your answer to ${bot}`,
    placeholder: "Write your answer",
    send: "Answer",
    failed: "Could not send. Try again, or open it in Botloft.",
  },
};
