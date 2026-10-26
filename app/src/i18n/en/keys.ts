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
  pasted: (names: string[], bot: string) =>
    `${names.length === 1 ? `The key ${names[0]} is` : `The keys ${names.join(", ")} are`} already kept, so ${bot} got the name in place of the value.`,
};
