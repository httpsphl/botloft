// What the search in Settings looks through: each setting's name and the
// page it is on, in the words of the current language.

import type { useT } from "../../i18n";
import { SYSTEM } from "../../lib/system";

export type Page =
  | "general"
  | "appearance"
  | "chat"
  | "alerts"
  | "account"
  | "tools"
  | "backup"
  | "archived"
  | "about";

export interface Entry {
  page: Page;
  label: string;
}

/** Accents and case do not matter when looking. */
export const plain = (text: string) =>
  text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase().trim();

export function settingsIndex(t: ReturnType<typeof useT>): Entry[] {
  const s = t.account.settings;
  const entry = (page: Page, ...labels: string[]): Entry[] =>
    labels.map((label) => ({ page, label }));
  return [
    ...entry(
      "general",
      s.language,
      s.background,
      s.keepWorking,
      s.tray,
      s.startWithSystem(SYSTEM),
      s.openAtSignIn(SYSTEM),
      s.keepAwake,
    ),
    ...entry("appearance", s.theme, s.size, s.lessMotion),
    ...entry("chat", s.enterSends, s.followBot, s.approvalWait),
    ...entry("alerts", s.notifyNeeds, s.notifyDone, s.markReplies, s.sound, s.appSounds),
    ...entry("account", t.account.cloud.title, t.account.mobile.title, t.account.mobile.connect),
    ...entry("tools", s.tools),
    ...entry("backup", t.account.backup.exportTitle, t.account.backup.importTitle, s.copiesTitle),
    ...entry("archived", s.archived),
    ...entry("about", s.about, s.checkUpdates),
  ];
}

/** The entries that match `query`, the page's own name counting too. */
export function search(entries: Entry[], query: string, names: Record<Page, string>): Entry[] {
  const wanted = plain(query);
  if (wanted === "") {
    return [];
  }
  return entries.filter(
    (entry) => plain(entry.label).includes(wanted) || plain(names[entry.page]).includes(wanted),
  );
}
