// An agent's keys (spec 32): the card where an agent asks for one, the Keys
// part of its details and the word after the owner pastes a saved key.

export const keys = {
  card: {
    asks: (bot: string) => `${bot} needs a key`,
    why: "What for",
    saved: (name: string) => `You saved the key ${name}`,
    dismissed: (name: string) => `You did not give the key ${name}`,
    notNow: "Not now",
    notNowHint: (bot: string) => `Closes the request and tells ${bot} the key is not coming now.`,
    dismissFailed: "Could not close the request",
  },
  form: {
    name: "Name",
    nameHint: "Capital letters, digits and _, like GITHUB_TOKEN. The agent reads the key by it.",
    value: "Key",
    valueHint: "Kept on this computer only, out of the chat. Nobody can read it back here.",
    description: "Description (optional)",
    descriptionPlaceholder: "What it is for",
    show: "Show the key",
    hide: "Hide the key",
    save: "Save",
    cancel: "Cancel",
    saveFailed: "Could not save the key",
    restarts: (bot: string) => `${bot} restarts to get it, without losing the conversation.`,
  },
  details: {
    title: "Keys",
    none: (bot: string) =>
      `No keys yet. When ${bot} needs a token or a password for another service, keep it here instead of pasting it in the chat.`,
    howItWorks: "How it works",
    explain: (bot: string) =>
      `${bot} gets each key as a variable by its name, and Botloft swaps a key it sees in the chat for its name. It is not a guarantee: if ${bot} prints a key anyway, it ends up in the engine's own history. The best protection is a key that can do little.`,
    add: "Add a key",
    replace: (name: string) => `Replace ${name}`,
    remove: (name: string) => `Delete ${name}`,
    missing: "Value missing",
    missingHint:
      "The name came with a backup, the value did not. Replace it to give the key again.",
    changed: (when: string) => `changed ${when}`,
    removeTitle: (name: string) => `Delete the key ${name}?`,
    removeBody: (bot: string) =>
      `${bot} restarts without it. The key still works at the service it came from: to stop it there, revoke it at that service.`,
    removeConfirm: "Delete",
    removeFailed: "Could not delete the key",
  },
  scan: {
    title: "Keys pasted in the conversations",
    intro:
      "Botloft looks through what you and your agents wrote for text that looks like a key: tokens from GitHub, Slack, Stripe, OpenAI, Anthropic, Google, AWS and sign-in tokens. It shows only how each one starts and ends.",
    run: "Look for keys",
    again: "Look again",
    none: "No keys found in the conversations.",
    found: (count: number) => (count === 1 ? "1 key found" : `${count} keys found`),
    places: (count: number) => (count === 1 ? "in 1 place" : `in ${count} places`),
    keep: "Keep as a key",
    keepTitle: "Keep as a key of",
    agent: "Agent",
    goneAgent: "an agent that is gone",
    remove: "Delete from Botloft",
    removeTitle: (preview: string) => `Delete ${preview} from Botloft?`,
    removeBody:
      "Every place it shows up will read [chave removida]. This cannot be undone, and the key still works at the service it came from.",
    done: "Done",
    failed: "Could not look for keys",
    scrubFailed: "Could not change the conversations",
    reachTitle: "Change the key at the service it came from",
    reach:
      "It already went through places Botloft does not control. Botloft does not touch the engine's own conversation history (starting a new conversation with the agent leaves it behind) or cloud copies already made (the next one goes without the key; old ones are deleted in Account).",
  },
  pasted: (names: string[], bot: string) =>
    `${names.length === 1 ? `The key ${names[0]} is` : `The keys ${names.join(", ")} are`} already kept, so ${bot} got the name in place of the value.`,
};
