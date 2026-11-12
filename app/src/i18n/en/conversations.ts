// The conversations of an agent (spec 40.7).

export const conversations = {
  main: "Main",
  untitled: "New conversation",
  new: "New conversation",
  newHint: (bot: string) => `Start another conversation with ${bot}, beside this one`,
  newFailed: "Could not start a new conversation",
  current: "Conversation",
};
