// The home page (spec 15.1): what waits for the owner across the crews,
// then the crews.

export const home = {
  label: "Home",
  title: "Home",
  /** Under the title: what waits and what works, in a few words. */
  summary: (waiting: number, working: number) =>
    [
      waiting > 0 && `${waiting} ${waiting === 1 ? "thing waits" : "things wait"} for you`,
      working > 0 && `${working} working`,
    ]
      .filter(Boolean)
      .join(" · ") || "All quiet",
  needsYou: "Needs you",
  working: "Working now",
  replies: "New replies",
  crews: "Crews",
  /** What the crews did since midnight, counted by Botloft (spec 31.4). */
  today: {
    title: "Today",
    nothing: "Nothing done yet today.",
    tasks: "Tasks done",
    failed: (count: number) => `${count} failed`,
    requests: "Requests answered",
    files: "Files edited",
    messages: "Messages between agents",
    busiest: "Worked the most",
  },
  empty: {
    title: "Nothing waits for you",
    body: "When an agent asks for your OK or a decision, it shows up here, from every crew.",
  },
  request: {
    /** At the head of a request, under the agent's name. */
    seeAll: "See the request",
    openChat: (bot: string) => `Open the chat with ${bot}`,
    openInChat: "Open in the chat",
    /** A request too long to show here whole. */
    cut: "Too long to show here whole. To allow it, open it in the chat.",
    /** A request answered at the computer only, with everything it changes. */
    atComputer: "This one changes things for good, so you answer it in the chat.",
    /** Answered elsewhere (in the chat, on the phone) before this click. */
    gone: "It was already answered",
  },
  failed: {
    line: (count: number) =>
      count === 1 ? "1 message was not delivered" : `${count} messages were not delivered`,
    show: "See and retry",
  },
  signedOut: {
    line: (bot: string) => `${bot} stopped: its engine is not signed in`,
    open: "Open",
  },
};
