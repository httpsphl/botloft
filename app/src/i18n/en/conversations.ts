// The conversations of an agent (spec 40.7).

export const conversations = {
  main: "Main",
  untitled: "New conversation",
  new: "New conversation",
  newHint: (bot: string) => `Start another conversation with ${bot}, beside this one`,
  newFailed: "Could not start a new conversation",
  current: "Conversation",
  count: (n: number) => `${n} conversations`,
  listOf: (bot: string) => `Conversations with ${bot}`,
  unread: "new reply",
  menuOf: (title: string) => `Conversation ${title}`,
  rename: "Rename",
  renameTitle: "Rename the conversation",
  titleLabel: "Name",
  save: "Save",
  renameFailed: "Could not rename the conversation",
  archive: "Archive",
  archiveTitle: (title: string) => `Archive ${title}?`,
  archiveBody:
    "It stops and leaves the list. Anything still on its way to it goes to the main conversation.",
  archiveFailed: "Could not archive the conversation",
};
