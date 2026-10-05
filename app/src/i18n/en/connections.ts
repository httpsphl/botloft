// Connected tools (spec 25): tools of the owner's own that bots can use, in
// Settings and in each bot's details. Plain words: no "MCP", no "server".

export const connections = {
  title: "Connected tools",
  intro:
    "Tools of your own that your bots can use besides the ones Botloft gives them, like a LinkedIn reader or a system of your company. Each bot gets only the ones you turn on for it, and it still asks you before using them.",
  loadFailed: "Could not read the connected tools",
  none: "No tools connected yet.",
  add: "Connect a tool…",
  usedBy: (names: string) => `Used by ${names}`,
  unused: "No bot uses it yet",
  kinds: {
    stdio: "A program on this computer",
    http: "A tool at an address",
  } as Record<string, string>,
  edit: (name: string) => `Edit ${name}`,
  remove: (name: string) => `Remove ${name}`,
  adding: {
    title: "Connect a tool",
    paste: "Paste its settings",
    pasteHint: "The text from its .mcp.json file, or the settings that came with the tool.",
    name: "Name",
    nameHint: "Only needed when the settings you pasted come without a name.",
    purpose: "What is it for? (optional)",
    purposeHint: "Your bots read this to know when to use it.",
    found: (count: number) => (count === 1 ? "Found 1 tool." : `Found ${count} tools.`),
    program: (name: string) =>
      `${name} is a program. It will run on your computer with your permissions, outside the folders Botloft keeps for your bots. Only connect programs you trust.`,
    address: (name: string) =>
      `${name} is a tool at an address. What your bot sends to it leaves this computer.`,
    command: "What it runs",
    understood: "I understand and I trust it",
    connect: "Connect",
    connecting: "Connecting…",
    failed: "Could not connect the tool",
  },
  problems: {
    empty: "Paste the tool's settings first.",
    notJson: "That is not valid JSON. Paste the text exactly as it is in the file.",
    noServers: "There is no tool in this text.",
    needsName: "These settings have no name: write one below.",
    unsupported: (server: string) =>
      `${server} uses a kind of connection Botloft does not support yet.`,
    unknownField: (server: string, field: string) =>
      `${server} has a setting Botloft does not know: "${field}". Remove it and try again.`,
    badValue: (server: string, field: string) =>
      `${server} has something wrong in "${field}". Check it and try again.`,
  },
  editing: {
    title: (name: string) => `Edit ${name}`,
    note: "To change what it runs, its address or its passwords, remove it and connect it again.",
    save: "Save",
    failed: "Could not save the tool",
  },
  removing: {
    title: (name: string) => `Remove ${name}?`,
    text: "Bots that use it lose it and start again when they are free. What you allowed them to do with it for good is forgotten.",
    confirm: "Remove",
    failed: "Could not remove the tool",
  },
  bot: {
    title: "Connected tools",
    none: "No tools connected yet. You can connect them in Settings, under Connected tools.",
    toggle: (tool: string, bot: string) => `${bot} can use ${tool}`,
    failed: "Could not change the tools of this bot",
    reason: "Why",
  },
  states: {
    connected: "Connected",
    pending: "Connecting…",
    needs_auth: "Needs you to sign in",
    failed: "Did not connect",
  } as Record<string, string>,
};
