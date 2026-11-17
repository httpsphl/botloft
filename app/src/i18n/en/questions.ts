// Questions the bots ask the owner (spec 23) and notes they leave them
// (23.10): the card in a bot's chat and the inbox on the left. What the
// bots ask or tell is data and never passes through here.

export const questions = {
  card: {
    asks: (bot: string) => `${bot} asks`,
    /** The card's head in the question box, under the bot's name. */
    question: "Question",
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
    /** A note the bot left (spec 23.10): it asks nothing. */
    tells: (bot: string) => `${bot} left you a note`,
    /** The note card's head in the inbox, under the bot's name. */
    note: "Note",
    replyLabel: (bot: string) => `Your reply to ${bot}`,
    replyPlaceholder: "Reply, if you want",
    reply: "Reply",
    seen: "Seen",
    seenHint: (bot: string) => `Closes the note. Writing to ${bot} in its chat closes it too.`,
    replied: "You replied",
    wasSeen: "Seen",
    seenFailed: "Could not close the note",
  },
  box: {
    label: "Inbox",
    /** The entry on the left, with how many questions and notes wait. */
    open: (count: number) =>
      count === 0
        ? "Questions and notes from your agents"
        : count === 1
          ? "1 question or note waits for you"
          : `${count} questions and notes wait for you`,
    title: "Inbox",
    intro:
      "When an agent needs your decision, or has something you should know, it leaves it here and carries on. Your answer goes to it as a message.",
    emptyTitle: "Nothing is waiting for you",
    emptyBody:
      "Agents ask and tell you things here, also at night, in a routine. Answer whenever you can.",
    openChat: (bot: string) => `Open the chat with ${bot}`,
    inCrew: (crew: string) => `in ${crew}`,
  },
  /** The conversation-list line for a question the bot asked. */
  activity: (text: string) => `Question: ${text}`,
  /** The conversation-list line for a note the bot left. */
  noteActivity: (text: string) => `Note: ${text}`,
};
