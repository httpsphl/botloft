// A bot's chat: the messages, what the bot did in each turn, approvals,
// files and the composer. Bot names, tool names and what the bots write
// are data and never pass through here.

/** A turn's tokens, already written as numbers. */
export interface TurnTokens {
  read: string;
  wrote: string;
}

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
    goneBot: "A bot no longer in the crew",
    routine: "Show what the routine asks",
  },
  run: {
    working: "Working",
    done: (time: string, tokens: string | null) =>
      `Done in ${time}${tokens === null ? "" : ` · ${tokens} tokens`}`,
    took: (time: string, tokens: TurnTokens | null) =>
      `Took ${time}${
        tokens === null ? "" : `. Read ${tokens.read} tokens and wrote ${tokens.wrote}.`
      }`,
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
    /** A finished group of calls in one line: "Ran 4 commands, read 2 files". */
    group: {
      command: (n: number) => (n === 1 ? "ran 1 command" : `ran ${n} commands`),
      read: (n: number) => (n === 1 ? "read 1 file" : `read ${n} files`),
      edit: (n: number) => (n === 1 ? "edited 1 file" : `edited ${n} files`),
      search: (n: number) =>
        n === 1 ? "searched the files once" : `searched the files ${n} times`,
      web: (n: number) => (n === 1 ? "read 1 web page" : `read ${n} web pages`),
      browser: (n: number) =>
        n === 1 ? "took 1 step in the browser" : `took ${n} steps in the browser`,
      message: (n: number) => (n === 1 ? "sent 1 message" : `sent ${n} messages`),
      other: (n: number) => (n === 1 ? "used 1 tool" : `used ${n} tools`),
      failed: (n: number) => `${n} failed`,
    },
  },
  /** A bot wrote down what it learned (spec 15.3), before the name. */
  memory: {
    bot: "Updated memory for",
    crew: "Updated crew memory for",
    file: "memory",
    changed: "What changed",
    now: "The memory now says:",
  },
  /** Replying to something in the chat (spec 9.3). */
  reply: {
    action: "Reply",
    to: (who: string) => `Reply to ${who}`,
    replyingTo: (who: string) => `Replying to ${who}`,
    cancel: "Cancel the reply",
    jump: "Show what this replies to",
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
    /** "Allow always" (spec 10.1), by what it covers. */
    always: {
      command: "Always allow this command",
      site: (site: string) => `Always allow ${site}`,
      file: "Always allow this file",
      tool: "Always allow",
      hint: (bot: string) =>
        `${bot} will not ask for this again. You can undo it in ${bot}'s details.`,
    },
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
  effort: {
    title: "Effort",
    button: (level: string) => `Effort: ${level}`,
    short: "Effort",
    faster: "Faster",
    smarter: "Smarter",
    names: {
      low: "Low",
      medium: "Medium",
      high: "High",
      xhigh: "Extra high",
      max: "Max",
    },
    hints: {
      low: (bot: string) => `${bot} answers fastest and thinks the least, for simple tasks`,
      medium: (bot: string) => `${bot} balances speed and thinking, good for most work`,
      high: (bot: string) => `${bot} thinks more before answering, for harder tasks`,
      xhigh: (bot: string) => `${bot} thinks a lot more, for long and complex work`,
      max: (bot: string) =>
        `${bot} thinks as much as it can: slowest, and uses the most of your plan`,
    },
    recommended: "Recommended",
    recommendedFor: (model: string) => `Recommended for ${model}`,
    useRecommended: "Use recommended",
    unknown: (bot: string) => `${bot} uses the level recommended for its model`,
    unavailable: "Not available",
    none: (bot: string, model: string) =>
      `${model} has no effort levels: ${bot} always answers at the same pace.`,
    thisModel: "This model",
    cost: "More effort uses up your plan's limit faster.",
    failed: "Could not change the effort",
    later: (bot: string, level: string) =>
      `${bot} switches to ${level} effort when it finishes what it's doing.`,
  },
  context: {
    title: "Conversation space",
    button: (used: string, total: string, percent: number) =>
      `Conversation space: ${used} of ${total} used (${percent}%)`,
    used: (used: string, total: string, percent: number) => `${used} / ${total} (${percent}%)`,
    about: (bot: string) =>
      `Everything ${bot} read and wrote in this conversation takes up space. The fuller it is, the more of your plan each message uses.`,
    autoLeft: (left: string, at: string) => `${left} left before it compacts by itself, at ${at}.`,
    autoNow: "It is full enough to compact by itself on the next message.",
    noAuto: "It does not compact by itself.",
    compact: "Compact now",
    compactHint: (bot: string) =>
      `Replaces what came before with a summary, so ${bot} has room again and each message uses less.`,
    compacting: "Compacting…",
    later: (bot: string) => `${bot} compacts the conversation when it finishes what it's doing.`,
    notRunning: (bot: string) => `${bot} is not running, so it cannot compact now.`,
    failed: "Could not compact the conversation",
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
  routineRequest: {
    title: (bot: string) => `${bot} wants to set up a routine`,
    titleFor: (bot: string, runner: string) => `${bot} wants to set up a routine for ${runner}`,
    explain: (runner: string) =>
      `A routine makes ${runner} work on its own at set times. It shows in the Routines tab, where you can pause or delete it.`,
    noteLabel: (bot: string) => `What to tell ${bot} if you say no`,
    notePlaceholder: (bot: string) => `If you say no, tell ${bot} why (optional)`,
    create: "Create routine",
    decline: "Not now",
    createFailed: "Could not create the routine",
    declineFailed: "Could not send the answer",
    created: (name: string) => `You created the routine ${name}`,
    declined: (name: string) => `You said no to the routine ${name}`,
    expired: (name: string) => `The routine ${name} was not answered in time`,
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
    compacted:
      "The conversation was compacted: what came before is now a summary, and there is room again.",
    autoCompacted:
      "The conversation was full, so it was compacted: what came before is now a summary.",
    compactFailed: (detail: string) => `The conversation could not be compacted: ${detail}`,
  },
};
