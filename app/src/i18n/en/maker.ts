// The Maker (spec 36): its chat, its plans and its settings.

export const maker = {
  emptyTitle: "Tell me what you do, and I build your crews",
  emptyBody:
    "Describe your work in a few lines: what you do every day, what repeats, what you would hand off. I put together crews, agents and routines ready to use, and you check all of it before anything is made.",
  startHere: "Let the Maker build",
  startHereHint: "Tell it about your work and it builds the crews for you.",
  card: {
    title: (title: string) => `The Maker wants to build: ${title}`,
    why: "Why",
    newCrew: "New crew",
    include: (name: string) => `Include ${name}`,
    goal: "Goal",
    goalHint: "The chief's instructions.",
    memory: "Crew memory",
    memoryHint: "Every agent of the crew reads this.",
    chief: (name: string) => `${name}, the chief`,
    agents: "Agents",
    routines: "Runs on its own",
    forAgent: (agent: string) => `For ${agent}`,
    runs: (count: number) =>
      count === 1
        ? "1 routine will run on its own. Each run uses a little of your plan."
        : `${count} routines will run on their own. Each run uses a little of your plan.`,
    nothingLeft: "Leave at least one crew in the plan.",
    build: "Build",
    decline: "Not now",
    noteLabel: "What to tell the Maker if you say no",
    notePlaceholder: "If you say no, tell the Maker why (optional)",
    buildFailed: "Could not build the plan",
    declineFailed: "Could not send the answer",
    built: (title: string) => `You built: ${title}`,
    builtAlone: (title: string) => `The Maker built: ${title}`,
    declined: (title: string) => `You did not build: ${title}`,
    expired: (title: string) => `${title} was not answered in time`,
    open: "Open",
    undo: "Undo",
    undoTitle: "Undo what the Maker built?",
    undoBody: (count: number) =>
      count === 1
        ? "The crew goes to the archive with its agents and routines. You can bring it back from Settings > Archived."
        : `The ${count} crews go to the archive with their agents and routines. You can bring them back from Settings > Archived.`,
    undone: "Undone: the crews are in the archive",
    undoFailed: "Could not undo it",
  },
  settings: {
    title: "The Maker",
    show: "Show the Maker",
    showOn: "It sits at the top of the list on the left.",
    showOff: "Hidden from the list. Nothing of it is deleted.",
    withoutAsking: "Let the Maker build without asking",
    withoutAskingOn: "It builds its plans at once; you can undo each one for a week.",
    withoutAskingOff: "You check each plan before anything is made.",
    confirmTitle: "Let the Maker build without asking?",
    confirmBody:
      "It will make crews, agents and routines as soon as it plans them, without showing you first. Archiving or pausing an agent still asks you, and you can undo each build for a week.",
    confirm: "Let it build",
  },
};
