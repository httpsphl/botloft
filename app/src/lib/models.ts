// Names for the Claude model ids Claude Code reports (spec 7.4).

const FAMILY = /^claude-(fable|opus|sonnet|haiku)-(\d+)(?:-(\d{1,2}))?(?:-\d{8})?$/;

/** "claude-opus-5-5" -> "Opus 5.5"; an id it does not know stays as it is. */
export function modelName(id: string): string {
  const match = FAMILY.exec(id);
  if (!match) {
    return id;
  }
  const [, family = "", major, minor] = match;
  const name = family.charAt(0).toUpperCase() + family.slice(1);
  return minor === undefined ? `${name} ${major}` : `${name} ${major}.${minor}`;
}

/** The family of a model id ("opus"), if it is one of Claude's. */
export function modelFamily(id: string): string | null {
  return FAMILY.exec(id)?.[1] ?? null;
}
