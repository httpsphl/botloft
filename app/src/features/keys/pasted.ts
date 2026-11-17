// Finding the keys the daemon swapped for their names in what the owner
// sent (spec 32.6): `[chave NAME]` in the message that went out and not in
// what was typed.

const MARK = /\[chave ([A-Z][A-Z0-9_]*)\]/g;

function names(text: string): Set<string> {
  return new Set([...text.matchAll(MARK)].map((match) => match[1] ?? ""));
}

/** The keys swapped in `sent`, in order, each once. */
export function pastedKeys(typed: string, sent: string): string[] {
  const before = names(typed);
  return [...names(sent)].filter((name) => name && !before.has(name));
}
