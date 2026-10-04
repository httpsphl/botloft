// Messages between bots and the owner: the timeline, writing one, and
// where each is on its way. Plain words only: a message is delivered,
// waiting or not delivered, never a "lease" or a "dead letter".

export const messages = {
  /** Bots calling each other, live. */
  calls: {
    label: "Bots calling each other",
    /** Before the call, for screen readers: the mascots show it. */
    caller: (bot: string) => `${bot}:`,
    calling: (bot: string) => `Calling ${bot}`,
    picked: (bot: string) => `${bot} picked it up`,
  },
  composer: {
    messageTo: "Message to",
    recipient: "Recipient",
    placeholder: (handle: string | undefined) => `Write to @${handle ?? "bot"}. Ctrl+Enter sends.`,
    send: "Send",
  },
  delivery: {
    delivered: "Delivered",
    read: "Read",
    readTitle: "The bot began working on it",
    delivering: "Delivering",
    waiting: "Waiting for the bot",
    retrying: (when: string, attempts: number) =>
      `Retrying ${when}, after ${attempts} failed ${attempts === 1 ? "try" : "tries"}`,
    notDelivered: "Not delivered",
    retry: "Retry",
    retryFailed: "Could not retry the delivery",
  },
  failed: {
    button: (count: number) => `${count} not delivered`,
    title: "Messages not delivered",
    retryAll: "Retry all",
    explanation:
      "Botloft stopped trying after several attempts. Retrying puts a message back in line; it goes out when the bot is ready.",
    to: (name: string) => `To ${name}`,
    tries: (count: number) => `${count} ${count === 1 ? "try" : "tries"}`,
  },
  row: {
    you: "You",
    system: "Botloft",
    goneBot: "a bot no longer in the crew",
    to: "to",
    task: "Task",
    result: "Result",
    due: (when: string) => `due ${when}`,
    taskStatus: {
      open: "open",
      done: "done",
      failed: "failed",
      cancelled: "cancelled",
      expired: "expired",
    },
  },
  timeline: {
    list: "Messages",
    loadFailed: "Could not load messages",
    loadOlder: "Load older messages",
    loading: "Loading messages…",
  },
};
