// Searching the chats (spec 8.8). What the bots and the owner wrote is
// data and never passes through here.

export const search = {
  label: "Search",
  /** The button on the left. */
  open: "Search the chats (Ctrl+K)",
  field: "Search the chats",
  placeholder: "Find words in your chats",
  scope: "Where to search",
  allCrews: "All crews",
  hint: "Finds words in what you, other agents and Botloft wrote to your agents, in what they replied and in their questions.",
  tooShort: "Type at least 2 letters",
  results: "Results",
  nothing: (query: string) => `Nothing found for “${query}”`,
  more: "Show more",
  failed: "Could not search",
  searching: "Searching…",
  inCrew: (crew: string) => `in ${crew}`,
  /** Who wrote what was found. */
  you: "You",
  botloft: "Botloft",
  goneBot: "An agent no longer in the crew",
  openHere: (bot: string) => `Open the chat with ${bot} here`,
};
