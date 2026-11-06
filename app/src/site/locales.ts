// The site's languages (spec 39.2): each is a page of its own, made when
// the site is built, at an address of its own.

import { site as en, type SiteMessages } from "../i18n/en/site";
import { site as es } from "../i18n/es/site";
import { site as ptBR } from "../i18n/pt-BR/site";

export type SiteLocale = "en" | "pt-BR" | "es";

/** Where the site lives, for the addresses that must be whole (search engines, link previews). */
export const SITE_URL = "https://botloft.comitium.com.br";
/** The code, the releases and the documents the site points to. */
export const REPO_URL = "https://github.com/httpsphl/botloft";
/** Claude Code's own setup page. */
export const CLAUDE_SETUP_URL = "https://code.claude.com/docs/en/setup";

export const SITE_LOCALES: readonly {
  id: SiteLocale;
  /** The address of its page. */
  path: string;
  /** Its own name, as a visitor looks for it. */
  name: string;
  /** The short name on the language switch. */
  short: string;
  /** For link previews (Open Graph). */
  og: string;
}[] = [
  { id: "en", path: "/", name: "English", short: "EN", og: "en_US" },
  { id: "pt-BR", path: "/pt-BR/", name: "Português (Brasil)", short: "PT", og: "pt_BR" },
  { id: "es", path: "/es/", name: "Español", short: "ES", og: "es_ES" },
];

const MESSAGES: Record<SiteLocale, SiteMessages> = { en, "pt-BR": ptBR, es };

export function messagesOf(locale: SiteLocale): SiteMessages {
  return MESSAGES[locale];
}

/** The language a page address stands for; any other address is English. */
export function localeOfPath(path: string): SiteLocale {
  const found = SITE_LOCALES.find(
    (locale) => locale.id !== "en" && (path === locale.path || `${path}/` === locale.path),
  );
  return found?.id ?? "en";
}

/** Where the choice of language is kept, so the English page sends no one back. */
export const LOCALE_KEY = "botloft.site.locale";
