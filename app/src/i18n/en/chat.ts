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
  },
  approval: {
    asks: (bot: string, tool: string) => `${bot} asks to use ${tool}`,
    wants: (bot: string, tool: string) => `${bot} wants to use ${tool}`,
    fullInput: "Full input",
    noteLabel: (bot: string) => `Note for ${bot} if you deny`,
    notePlaceholder: "Why not? Sent to the bot if you deny (optional)",
    allow: "Allow",
    deny: "Deny",
    allowFailed: "Could not allow it",
    denyFailed: "Could not deny it",
    allowed: (tool: string) => `You allowed ${tool}`,
    denied: (tool: string) => `You denied ${tool}`,
    expired: (tool: string) => `${tool} was not approved in time`,
  },
  markdown: {
    image: "image",
    openLinkFailed: "Could not open the link",
  },
  notice: {
    signedOut:
      "Claude Code is not signed in, or the account can't be used right now. Sign in to Claude, or check your Claude plan.",
    usageLimit: "Your Claude plan reached its usage limit. Messages wait until it resets.",
    turnFailed: (detail: string) => `The bot couldn't finish this: ${detail}`,
  },
};
