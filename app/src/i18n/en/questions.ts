// Questions the bots ask the owner (spec 23): the card in a bot's chat and
// the question box on the left. What the bots ask is data and never passes
// through here.

export const questions = {
  card: {
    asks: (bot: string) => `${bot} asks`,
    pick: "Pick an answer",
    answerLabel: (bot: string) => `Your answer to ${bot}`,
    placeholder: "Write your answer",
    send: "Answer",
    dismiss: "Dismiss",
    dismissHint: (bot: string) =>
      `Closes the question without telling ${bot}. To let it know, answer instead.`,
    answered: "You answered",
    dismissed: "Dismissed without an answer",
    answerFailed: "Could not send the answer",
    dismissFailed: "Could not dismiss the question",
  },
  box: {
    label: "Questions",
    /** The entry on the left, with how many questions wait. */
    open: (count: number) =>
      count === 0
        ? "Questions from your bots"
        : count === 1
          ? "1 question waits for you"
          : `${count} questions wait for you`,
    title: "Questions from your bots",
    intro:
      "When a bot needs your decision to go on, it asks here and carries on. Your answer goes to it as a message.",
    emptyTitle: "No question is waiting for you",
    emptyBody:
      "Bots ask here when they need you, also at night, in a routine. Answer whenever you can.",
    openChat: (bot: string) => `Open the chat with ${bot}`,
    inCrew: (crew: string) => `in ${crew}`,
  },
  /** The conversation-list line for a question the bot asked. */
  activity: (text: string) => `Question: ${text}`,
};
