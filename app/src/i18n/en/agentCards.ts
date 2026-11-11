// Cards an agent shows in its chat (spec 8.11): tables, charts, buttons
// and a form. What the agent writes in them is data and never passes
// through here.

export const agentCards = {
  /** The card's region, for screen readers. */
  label: (bot: string, title: string) => `${bot}'s card: ${title}`,
  /** Under the title once the agent showed it again. */
  updated: (time: string) => `Updated ${time}`,
  send: "Send",
  required: "required",
  pick: "Pick one",
  /** What the owner did, once the version is answered. */
  pressed: (label: string) => `You chose “${label}”`,
  sent: "You sent the form",
  waiting: (bot: string) => `${bot} has your answer. To ask again, it shows the card again.`,
  failed: "Could not send. Try again.",
  chart: {
    /** The chart's description for screen readers. */
    label: (kind: string, series: string) => `${kind} chart of ${series}`,
    bar: "Bar",
    line: "Line",
    asTable: "Show as a table",
    asChart: "Show as a chart",
    label0: "Label",
  },
  /** The conversation-list line for a card. */
  activity: (title: string) => `Card: ${title}`,
};
