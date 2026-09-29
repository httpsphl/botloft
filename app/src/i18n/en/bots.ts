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
      hint: "Claude Code is not signed in, or the account can't be used. Sign in to Claude and the bot starts again by itself. Already signed in? Check your Claude plan, then restart the bot.",
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
    hint: (crew: string) => `Leads ${crew}: plans the work and suggests new bots`,
  },
  header: {
    noRole: "No role",
    pause: "Pause",
    resume: "Resume",
    restart: "Restart",
    showDetails: "Show details",
    hideDetails: "Hide details",
    more: "More bot actions",
    edit: "Edit",
    restartFresh: "Restart with a new conversation",
    openFolder: "Open folder",
    makeChief: "Make crew chief",
    stopChief: "Stop being chief",
    archive: "Archive bot",
    failed: {
      pause: "Could not pause the bot",
      resume: "Could not resume the bot",
      restart: "Could not restart the bot",
      openFolder: "Could not open the folder",
      chief: "Could not change the chief",
      archive: "Could not archive the bot",
    },
    fresh: {
      title: "Start a new conversation?",
      confirm: "Restart",
      body: (name: string) =>
        `${name} restarts without its current conversation. Its folder and its CLAUDE.md stay as they are.`,
    },
    archiveConfirm: {
      title: (name: string) => `Archive ${name}?`,
      confirm: "Archive bot",
      body: "The bot stops and leaves the crew. Messages still waiting for it are not delivered.",
    },
  },
  dialog: {
    newTitle: "New bot",
    editTitle: (name: string) => `Edit ${name}`,
    create: "Create bot",
    save: "Save",
    name: "Name",
    namePlaceholder: "Reviewer",
    nameHint: "Other bots reach it by the handle made from this name.",
    role: "Role",
    rolePlaceholder: "Reviews pull requests before they merge",
    instructions: "Instructions",
    instructionsPlaceholder: "How this bot works, what it may do on its own and when to ask.",
    instructionsHint: "Saved to the bot's rules now; the bot reads them the next time it starts.",
    color: "Color",
    swatch: (color: string) => `Color ${color}`,
    colorUnset: "Left unset, the bot gets the crew's next color.",
  },
  notices: {
    crewPaused: (crew: string) => `${crew} is paused`,
    crewPausedBody: "Its bots stay stopped until you resume the crew.",
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
  },
};
