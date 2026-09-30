// A bot's chat: the messages, what the bot did in each turn, approvals,
// files and the composer. Bot names, tool names and what the bots write
// are data and never pass through here.

export const chat = {
  view: {
    label: (bot: string) => `Chat with ${bot}`,
    loadFailed: "Could not load the chat",
    loadEarlier: "Load earlier messages",
    loading: "Loading the chat…",
    emptyTitle: (bot: string) => `Start a conversation with ${bot}`,
    emptyBody: "Ask for anything its folder and tools can do. You can attach files and images too.",
    messages: "Messages",
    dropToAttach: "Drop to attach to your message",
  },
  composer: {
    paused: (bot: string) => `${bot} is paused. What you send waits until it runs again.`,
    filesToSend: "Files to send",
    remove: (file: string) => `Remove ${file}`,
    label: (bot: string) => `Message to ${bot}`,
    placeholder: (bot: string) => `Message ${bot}`,
    attach: "Attach files",
    attachHint: "Attach files (or paste, or drop them on the chat)",
    tooLong: (length: number, max: number) => `${length}/${max} characters`,
    keys: "Enter to send, Shift+Enter for a new line",
    keysWithCtrl: "Ctrl+Enter to send, Enter for a new line",
    send: "Send",
  },
  files: {
    tooMany: (max: number) => `A message can carry up to ${max} files.`,
    tooBig: (size: string) => `Files in one message can add up to ${size}. Send them in parts.`,
    unreadable: (file: string) => `could not read ${file}`,
  },
  attachments: {
    label: "Attachments",
    file: "File",
    showInFolder: (file: string) => `Show ${file} in its folder`,
    showInFolderHint: "Show in folder",
    openFolderFailed: "Could not open the folder",
    missing: "No longer in the bot's folder",
    loading: (file: string) => `Loading ${file}`,
  },
  inbound: {
    task: "Task",
    result: "Result",
    archivedBot: "An archived bot",
  },
  run: {
    working: "Working",
    done: (time: string) => `Done in ${time}`,
    took: (time: string, usd: number | null) =>
      `Took ${time}${usd === null ? "" : ` · about $${usd.toFixed(2)} of usage`}`,
    stopped: (reason: string) => `The bot stopped working on this: ${reason}`,
  },
  tools: {
    label: "Tool calls",
    running: "Running",
    done: "Done",
    failed: "Failed",
    input: "Input",
    output: "Output",
    error: "Error",
    command: "Command",
    commandCut: "This command is too long to show in full.",
  },
  approval: {
    asks: (bot: string, action: string) => `${bot} asks to ${action}`,
    wants: (bot: string, action: string) => `${bot} wants to ${action}`,
    explainedBy: (bot: string) => `${bot} wrote this. What actually runs is the command below.`,
    unexplained: (bot: string) =>
      `${bot} did not say what this command is for. If you are not sure, deny it and ask.`,
    command: "See the command",
    noteLabel: (bot: string) => `Note for ${bot} if you deny`,
    notePlaceholder: "Why not? Sent to the bot if you deny (optional)",
    allow: "Allow",
    deny: "Deny",
    allowFailed: "Could not allow it",
    denyFailed: "Could not deny it",
    allowed: (action: string) => `Allowed: ${action}`,
    denied: (action: string) => `Denied: ${action}`,
    expired: (action: string) => `Not answered in time: ${action}`,
  },
  mode: {
    title: "Mode",
    button: (mode: string) => `Mode: ${mode}`,
    names: {
      auto: "Auto",
      default: "Manual",
      accept_edits: "Accept edits",
      plan: "Plan",
      bypass_permissions: "Bypass permissions",
    },
    hints: {
      auto: (bot: string) => `${bot} decides what needs your OK`,
      default: (bot: string) => `${bot} always asks before making changes`,
      accept_edits: (bot: string) => `${bot} edits files without asking`,
      plan: (bot: string) => `${bot} makes a plan before making changes`,
      bypass_permissions: (bot: string) => `${bot} does everything without asking`,
    },
    turnOn: "Turn on",
    failed: "Could not change the mode",
    later: (bot: string, mode: string) =>
      `${bot} switches to ${mode} when it finishes what it's doing.`,
    bypassTitle: (bot: string) => `Let ${bot} do anything without asking?`,
    bypassBody: (bot: string) =>
      `${bot} will edit files, run commands and use the internet on this computer without asking you first.`,
    bypassRisk:
      "It is not limited to its folder: it can read and change your other files, other bots' files and Botloft's own files. A message from someone else could get it to do something you did not want.",
    bypassChief: (bot: string) =>
      `${bot} leads its crew, so it will also create new bots without asking you.`,
    bypassAdvice:
      "Turn this on only for a bot you trust with everything, and only while you need it.",
    badge: "Asks nothing",
    badgeHint:
      "This bot does everything without asking. Change it in the mode picker below the chat.",
  },
  model: {
    title: "Model",
    button: (model: string) => `Model: ${model}`,
    names: {
      default: "Plan default",
      fable: "Fable",
      opus: "Opus",
      sonnet: "Sonnet",
      haiku: "Haiku",
    },
    short: "Default",
    hints: {
      default: (bot: string, inUse: string | null) =>
        inUse
          ? `${bot} uses your plan's default model, now ${inUse}`
          : `${bot} uses your plan's default model`,
      fable: (bot: string) => `${bot} is at its most capable, for the hardest work`,
      opus: (bot: string) => `${bot} handles long, complex tasks well`,
      sonnet: (bot: string) => `${bot} is fast and capable, good for most work`,
      haiku: (bot: string) => `${bot} is quickest and uses less of your plan, for simple tasks`,
    },
    cost: "More capable models use up your plan's limit faster.",
    failed: "Could not change the model",
    later: (bot: string, model: string) =>
      `${bot} switches to ${model} when it finishes what it's doing.`,
  },
  suggestion: {
    title: (bot: string) => `${bot} suggests a new bot`,
    why: "Why",
    name: "Name",
    role: "Role",
    model: "Model",
    instructions: "Instructions",
    startsNow: (bot: string) => `It starts right away and gets its work from ${bot}.`,
    noteLabel: (bot: string) => `What to tell ${bot} if you say no`,
    notePlaceholder: (bot: string) => `If you say no, tell ${bot} why (optional)`,
    create: "Create bot",
    decline: "Not now",
    createFailed: "Could not create the bot",
    declineFailed: "Could not send the answer",
    created: (name: string) => `You created ${name}`,
    declined: (name: string) => `You said no to ${name}`,
    expired: (name: string) => `${name} was not answered in time`,
  },
  plan: {
    ready: (bot: string) => `${bot} made a plan and wants to go ahead`,
    noteLabel: (bot: string) => `What ${bot} should change in the plan`,
    notePlaceholder: "What should change? Sent to the bot if you keep planning (optional)",
    approve: "Approve plan",
    keepPlanning: "Keep planning",
    approveFailed: "Could not approve the plan",
    keepFailed: "Could not send it back",
    approved: "You approved the plan",
    sentBack: "You asked for changes to the plan",
    expired: "The plan was not approved in time",
  },
  markdown: {
    image: "image",
    openLinkFailed: "Could not open the link",
  },
  notice: {
    signedOut:
      "Claude Code is not signed in, or the account can't be used right now. Sign in to Claude, or check your Claude plan.",
    usageLimit: "Your Claude plan reached its usage limit. Messages wait until it resets.",
    modelUnavailable:
      "This bot's model isn't available: it may not be on your Claude plan. Pick another model below the chat.",
    turnFailed: (detail: string) => `The bot couldn't finish this: ${detail}`,
  },
};
