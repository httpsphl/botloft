// What the owner started writing to each bot, so going to another chat and
// back does not lose it (spec 15.1). Kept in memory only, while the app is
// open: a message can carry personal data, and nothing of it goes to disk.

const drafts = new Map<string, string>();

export function draftOf(botId: string): string {
  return drafts.get(botId) ?? "";
}

export function keepDraft(botId: string, text: string): void {
  if (text) {
    drafts.set(botId, text);
  } else {
    drafts.delete(botId);
  }
}
