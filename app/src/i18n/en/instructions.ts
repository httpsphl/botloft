// Settings > Instructions for all (spec 7.10): what the owner writes for
// every agent to follow.

export const instructions = {
  page: "Instructions for all",
  label: "What every agent follows",
  intro: "Every agent reads this in every conversation, on any engine. Short is better.",
  placeholder:
    "For example: Answer me in English. Ask me before you delete files. Put reports in the Reports folder.",
  howItWorks: "How it works",
  explain:
    "When this disagrees with something else, the more specific wins: what you ask in the chat, then each agent's own instructions, and then these. They come before what an agent noted for itself and what another agent asks it, and they never skip an approval. Agents read them when they start: an available agent starts again now, in the same conversation, and one that is working does when it finishes.",
  save: "Save",
  saved: "Saved. Available agents start again to read it.",
  saveFailed: "Could not save the instructions",
  loadFailed: "Could not read the instructions",
};
