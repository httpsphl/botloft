// A bot's screen: its states, header and actions, the create and edit
// dialog and the details panel. Names, roles and instructions are the
// owner's own text and are never translated.

export const bots = {
  /** One entry per bot state, keyed like the protocol's `BotState`. */
  states: {
    offline: { label: "Offline", hint: "Not running." },
    launching: { label: "Starting", hint: "Claude Code is starting." },
    idle: { label: "Idle", hint: "Ready for work." },
    busy: { label: "Working", hint: "Working on something." },
    needs_approval: {
      label: "Needs approval",
      hint: "Waiting for you to allow or deny a tool in its chat.",
    },
    rate_limited: {
      label: "Usage limit",
      hint: "Your Claude plan hit its usage limit; messages wait until it resets.",
    },
    auth_error: {
      label: "Sign-in needed",
      hint: "Claude Code is not signed in, or the account can't be used. Sign in to Claude and the agent starts again by itself. Already signed in? Check your Claude plan, then restart the agent.",
    },
    backoff: {
      label: "Restarting",
      hint: "It stopped unexpectedly; Botloft starts it again shortly.",
    },
    archived: { label: "Archived", hint: "Archived." },
    /** Paused (the bot or its crew) and not running. */
    paused: { label: "Paused", hint: "Paused; resume to start it." },
  },
  chief: {
    badge: "Chief",
    hint: (crew: string) => `Leads ${crew}: plans the work and suggests new agents`,
  },
  header: {
    noRole: "No role",
    pause: "Pause",
    resume: "Resume",
    restart: "Restart",
    showDetails: "Show details",
    hideDetails: "Hide details",
    more: "More agent actions",
    menuOf: (name: string) => `Actions for ${name}`,
    edit: "Edit",
    restartFresh: "Restart with a new conversation",
    openFolder: "Open folder",
    makeChief: "Make crew chief",
    stopChief: "Stop being chief",
    markUnread: "Mark as unread",
    markRead: "Mark as read",
    archive: "Archive agent",
    delete: "Delete agent",
    failed: {
      pause: "Could not pause the agent",
      resume: "Could not resume the agent",
      restart: "Could not restart the agent",
      openFolder: "Could not open the folder",
      chief: "Could not change the chief",
      archive: "Could not archive the agent",
      delete: "Could not delete the agent",
    },
    fresh: {
      title: "Start a new conversation?",
      confirm: "Restart",
      body: (name: string) =>
        `${name} restarts without its current conversation. Its folder and its CLAUDE.md stay as they are.`,
    },
    archiveConfirm: {
      title: (name: string) => `Archive ${name}?`,
      confirm: "Archive agent",
      body: "The agent stops and leaves the crew. Messages still waiting for it are not delivered.",
    },
    deleteConfirm: {
      title: (name: string) => `Delete ${name}?`,
      confirm: "Delete agent",
      /** `running` is false for an archived bot, which stopped long ago. */
      removed: (name: string, running: boolean) =>
        `${name} ${running ? "stops now and leaves" : "leaves"} Botloft for good, with its conversation, its routines and the tasks it was part of. This can't be undone.`,
      chief: (crew: string) => `${crew} will be left without a chief.`,
      kept: (name: string) => `${name}'s folder stays on your computer, with everything in it:`,
      recycle: "Move this folder to the Recycle Bin",
      recycled: (name: string) =>
        `${name}'s folder goes to the Recycle Bin, where you can still get it back:`,
    },
  },
  dialog: {
    newTitle: "New agent",
    editTitle: (name: string) => `Edit ${name}`,
    create: "Create agent",
    save: "Save",
    name: "Name",
    namePlaceholder: "Reviewer",
    nameHint: "Other agents reach it by the handle made from this name.",
    role: "Role",
    rolePlaceholder: "Reviews pull requests before they merge",
    instructions: "Instructions",
    instructionsPlaceholder: "How this agent works, what it may do on its own and when to ask.",
    instructionsHint:
      "Saved to the agent's rules now; the agent reads them the next time it starts.",
    fullAccess: {
      title: "Run any command without asking",
      off: "Off: it may run only the commands listed below.",
      on: "On: it runs any command without asking you, and a command can read or change anything you can. Only for an agent you trust.",
    },
    commands: {
      title: "Commands this agent may run",
      placeholder: "git status\nnpm test",
      hint: "One command per line; it may run these and anything that starts with them, without asking you. It cannot ask, so everything else is refused. A single * lets it run anything. The agent restarts to use the list.",
    },
    agent: {
      title: "Runs on",
      names: {
        claude: "Claude Code",
        agy: "Antigravity (experimental)",
        codex: "Codex",
      },
      experimental:
        "Experimental. This agent cannot ask you before it acts. It can work with files in its own folders and use the crew's tools, but it cannot run commands, and it has no usage meter or model choice.",
      experimentalCodex:
        "Experimental. This agent asks you before it changes a file or runs a command, like Claude Code. It can still read any file.",
    },
    color: "Color",
    swatch: (color: string) => `Color ${color}`,
    colorUnset: "Left unset, the agent gets the crew's next color.",
    custom: "Pick any color",
    picker: {
      area: "Saturation and brightness",
      areaValue: (saturation: string, brightness: string) =>
        `Saturation ${saturation}, brightness ${brightness}`,
      hue: "Hue",
      hex: "Hex",
      channels: { r: "R", g: "G", b: "B" },
    },
  },
  notices: {
    crewPaused: (crew: string) => `${crew} is paused`,
    crewPausedBody: "Its agents stay stopped until you resume the crew.",
  },
  details: {
    title: (name: string) => `About ${name}`,
    close: "Close details",
    role: "Role",
    noRole: "No role yet.",
    folder: "Folder",
    process: "Process",
    notStarted: "not started",
    generation: (generation: number) => `generation ${generation}`,
    instructions: "Instructions",
    noInstructions: "None yet.",
    always: "Allowed without asking",
    alwaysNone: (bot: string) =>
      `Nothing yet. When ${bot} asks for something, "Always allow" puts it here.`,
    alwaysRemove: (what: string) => `Ask again: ${what}`,
    alwaysRemoveFailed: "Could not remove it",
    crews: "Other crews",
    crewsNone: (bot: string) =>
      `${bot} reaches only its own crew. When it asks for another one, "Always" puts it here.`,
    crewsWhole: (crew: string) => `The whole crew ${crew}`,
    crewsBot: (bot: string, crew: string) => `${bot}, of the crew ${crew}`,
    crewsRemove: (what: string) => `Take back: ${what}`,
    crewsRemoveFailed: "Could not take it back",
    crewsKinds: { talk: "talk", read: "read files", edit: "edit files" },
  },
};
