// Crews: the sidebar with its conversation list, a crew's page, creating
// and renaming a crew, and the tasks its bots give each other.

export const crews = {
  newCrew: "New crew",
  newBot: "New bot",
  rename: "Rename",
  paused: "Paused",
  sidebar: {
    label: "Crews",
    noMessages: "No messages yet",
    /** The conversation-list line for what the owner wrote. */
    fromOwner: (text: string) => `You: ${text}`,
    /** The conversation-list line for an approval the bot is waiting on. */
    awaitingApproval: (text: string) => `Waiting for approval: ${text}`,
  },
  dialog: {
    folder: "Work folder",
    folderHint: "Where the bots put what they make. It can be a folder you already use.",
    folderDefault: "A new folder inside Botloft",
    chooseFolder: "Choose folder…",
    pickTitle: "Choose where the crew works",
    useDefault: "Use a new folder",
    goal: "What is this crew for?",
    goalPlaceholder: "Build and keep up my bakery's website",
    goalHint:
      "The crew starts with a chief, who reads this, plans the work and suggests the bots it needs.",
    chiefModel: "Chief's model",
    chiefName: "Chief",
    chiefRole: "Leads the crew: plans the work, suggests new bots and hands out tasks",
    renameTitle: "Rename crew",
    create: "Create crew",
    name: "Name",
    namePlaceholder: "Research",
    createHint: "Bots in a crew can message each other and share a folder.",
    renameHint: (slug: string) => `The folder keeps its name (${slug}).`,
  },
  view: {
    bots: (count: number) => (count === 1 ? "1 bot" : `${count} bots`),
    pausedNote: "paused: its bots stay stopped until you resume it",
    pause: "Pause crew",
    resume: "Resume crew",
    moreActions: "More crew actions",
    folder: (path: string) => `Works in ${path}`,
    openFolder: "Open work folder",
    changeFolder: "Change work folder…",
    moveTitle: (crew: string) => `Move ${crew} to another folder?`,
    moveBody: (path: string) =>
      `The bots will work in ${path}. Each one restarts when it finishes what it's doing. Files already made stay where they are.`,
    move: "Move",
    archive: "Archive crew",
    tabs: {
      label: "Crew views",
      bots: "Bots",
      timeline: "Timeline",
      tasks: "Tasks",
    },
    timelineEmpty:
      "No messages yet. Write to a bot below; what the bots send each other shows up here too.",
    archiveTitle: (crew: string) => `Archive ${crew}?`,
    archiveBody: (bots: number) =>
      bots === 0
        ? "The crew leaves the app."
        : `The crew leaves the app. ${bots === 1 ? "Its bot stops" : `Its ${bots} bots stop`}, and messages still waiting for them are not delivered.`,
    failed: {
      pause: "Could not pause the crew",
      resume: "Could not resume the crew",
      changeFolder: "Could not change the folder",
      openFolder: "Could not open the folder",
      archive: "Could not archive the crew",
    },
  },
  bots: {
    empty: (crew: string) => `No bots in ${crew} yet.`,
    emptyHint: "A bot is a Claude Code session that keeps running, with its own folder and role.",
    noRole: "No role yet.",
  },
  tasks: {
    show: "Show",
    open: "Open",
    all: "All",
    list: "Tasks",
    noOpen: "No open tasks. Bots create tasks for each other with send_message.",
    none: "No tasks yet. Bots create tasks for each other with send_message.",
    status: {
      open: "Open",
      done: "Done",
      failed: "Failed",
      cancelled: "Cancelled",
      expired: "Expired",
    },
    archivedBot: "an archived bot",
    /** Read between the requester and the assignee. */
    asked: "asked",
    hop: (hops: number) => `hop ${hops}`,
    hopHint: "Position in a chain of delegations",
    /** `time` is relative, like "in 5 min" or "2 hr ago". */
    due: (time: string) => `due ${time}`,
    overdue: (time: string) => `overdue, was due ${time}`,
    /** `status` is the status label; `time` is when it ended. */
    ended: (status: string, time: string) => `${status.toLowerCase()} ${time}`,
  },
};
