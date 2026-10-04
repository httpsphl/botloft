// Crews: the sidebar with its conversation list, a crew's page, creating
// and renaming a crew, and the tasks its bots give each other.

export const crews = {
  newCrew: "New crew",
  newBot: "New bot",
  rename: "Rename",
  paused: "Paused",
  sidebar: {
    /** The right-click menu of a crew in the list. */
    menuOf: (crew: string) => `Actions for ${crew}`,
    label: "Crews",
    noMessages: "No messages yet",
    /** The conversation-list line for what the owner wrote. */
    fromOwner: (text: string) => `You: ${text}`,
    /** The conversation-list line for an approval the bot is waiting on. */
    awaitingApproval: (text: string) => `Waiting for approval: ${text}`,
    unread: "unread",
    /** Beside a crew's name: how many of its bots are unread. */
    unreadCount: (count: number) => `${count} unread`,
    showAll: "See all crews",
    collapseAll: "Fold all crews",
    expandAll: "Unfold all crews",
    /** The arrow beside a crew's name, which hides or shows its bots. */
    collapse: (crew: string) => `Fold ${crew}`,
    expand: (crew: string) => `Unfold ${crew}`,
    /** Beside a folded crew's name, when one of its bots needs the owner. */
    waiting: "a bot needs you",
  },
  /** The page of all crews, opened from "Crews" on the left. */
  overview: {
    count: (count: number) => (count === 1 ? "1 crew" : `${count} crews`),
    working: (count: number) => `${count} working`,
    waiting: (count: number) => (count === 1 ? "1 needs you" : `${count} need you`),
    calm: "All quiet",
    noBots: "No bots yet",
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
    openFolderHint: "Click to open it",
    changeFolder: "Change work folder…",
    moveTitle: (crew: string) => `Move ${crew} to another folder?`,
    moveBody: (path: string) =>
      `The bots will work in ${path}. Each one restarts when it finishes what it's doing. Files already made stay where they are.`,
    move: "Move",
    archive: "Archive crew",
    delete: "Delete crew",
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
    deleteTitle: (crew: string) => `Delete ${crew}?`,
    /** `running` is false for an archived crew, which stopped long ago. */
    deleteBody: (crew: string, bots: number, running: boolean) =>
      bots === 0
        ? `${crew} leaves Botloft for good. This can't be undone.`
        : `${crew} and ${bots === 1 ? "its bot" : `its ${bots} bots`} ${running ? "stop now and leave" : "leave"} Botloft for good, with their conversations, routines and tasks. This can't be undone.`,
    deleteKept:
      "The folders stay on your computer, with everything in them: each bot's own folder and the crew's work folder:",
    deleteRecycle: "Move the crew's folders to the Recycle Bin",
    deleteRecycled:
      "The crew's folder goes to the Recycle Bin, with each bot's folder and the work folder in it. You can still get it back from there:",
    deleteRecycledChosen:
      "Each bot's own folder goes to the Recycle Bin, where you can still get it back. The work folder you chose stays where it is:",
    failed: {
      pause: "Could not pause the crew",
      resume: "Could not resume the crew",
      changeFolder: "Could not change the folder",
      openFolder: "Could not open the folder",
      archive: "Could not archive the crew",
      delete: "Could not delete the crew",
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
