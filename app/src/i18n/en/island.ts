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
    chat: "Chat",
    usage: "Usage",
  },
  /** On the pill, once the 5-hour window passes 80%. */
  planUsed: (percent: number) => `plan at ${percent}%`,
  chat: {
    pick: "Pick an agent",
    files: "Files to send",
    removeFile: (name: string) => `Remove ${name}`,
    label: (bot: string) => `Message to ${bot}`,
    placeholder: (bot: string) => `Write to ${bot}, or drop a file here`,
    send: "Send",
  },
  usage: {
    plan: "Claude plan",
    routines: "Next routines",
    noRoutines: "No routine coming up.",
    runNow: "Run now",
    started: "Started",
  },
  openApp: "Open Botloft",
  hide: "Hide",
  engines: { claude: "Claude Code", agy: "Antigravity", codex: "Codex" },
  /** An agent at work that has not used a tool yet in this turn. */
  thinking: "Thinking",
  moreWorking: (count: number) => (count === 1 ? "+1 more working" : `+${count} more working`),
  /** The cheer, when an agent finishes. */
  finished: (bot: string) => `${bot} finished`,
  today: {
    title: "Today",
    since: "since midnight",
    tasks: (count: number): string => (count === 1 ? "task" : "tasks"),
    requests: (count: number): string => (count === 1 ? "request" : "requests"),
    /** Under the lines added, with how many files. */
    lines: (files: number) => (files === 1 ? "lines, 1 file" : `lines, ${files} files`),
    busiest: "worked the most",
  },
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
