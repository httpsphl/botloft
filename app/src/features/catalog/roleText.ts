// What the owner reads about a role (spec 26.5): the app's text in the
// owner's language, by the role's id. A role the app has no text for (a
// daemon newer than the app) shows the daemon's English.

import type { Messages } from "../../i18n";
import type { BotTemplate } from "../../lib/protocol.gen";

type Known = Messages["catalog"]["roles"]["developer"];

export interface RoleText {
  name: string;
  role: string;
  summary: string;
  /** `null` for a role the app has no text for. */
  about: string | null;
  when: string | null;
  pairs: string | null;
}

export function roleText(t: Messages, template: BotTemplate): RoleText {
  const known = (t.catalog.roles as Record<string, Known | undefined>)[template.id];
  return (
    known ?? {
      name: template.name,
      role: template.role,
      summary: template.summary,
      about: null,
      when: null,
      pairs: null,
    }
  );
}
