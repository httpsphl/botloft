// The app's languages (spec 15.6). Every string the owner reads lives in
// `i18n/<locale>/`, one file per area; English is the reference and its
// shape is the `Messages` type, so a missing or extra key in another
// language does not compile. Text that depends on values is a function.
//
// Components read the current messages with `useT()`; code outside React
// (toasts, formatting) reads them with `t()`. The owner's choice is kept
// in localStorage; "system" follows Windows' display language.

import { useStore } from "zustand";
import { useShallow } from "zustand/react/shallow";
import { createStore } from "zustand/vanilla";
import { en, type Messages } from "./en";
import { es } from "./es";
import { ptBR } from "./pt-BR";

export type { Messages } from "./en";

export type Locale = "en" | "pt-BR" | "es";
export type LocaleChoice = Locale | "system";

/** Each language by its own name, as the owner looks for it. */
export const LOCALES: readonly { id: Locale; name: string }[] = [
  { id: "en", name: "English" },
  { id: "pt-BR", name: "Português (Brasil)" },
  { id: "es", name: "Español" },
];

const CATALOG: Record<Locale, Messages> = { en, "pt-BR": ptBR, es };
const STORAGE_KEY = "botloft.locale";

/** The first supported language among the system's preferred ones. */
export function systemLocale(
  languages: readonly string[] = typeof navigator === "undefined"
    ? []
    : (navigator.languages ?? [navigator.language]),
): Locale {
  for (const tag of languages) {
    const lower = tag.toLowerCase();
    if (lower.startsWith("pt")) {
      return "pt-BR";
    }
    if (lower.startsWith("es")) {
      return "es";
    }
    if (lower.startsWith("en")) {
      return "en";
    }
  }
  return "en";
}

function readChoice(): LocaleChoice {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "system" || (saved && saved in CATALOG)) {
      return saved as LocaleChoice;
    }
  } catch {
    // Storage can be off; the system language is a fine default.
  }
  return "system";
}

interface I18nState {
  choice: LocaleChoice;
  locale: Locale;
  messages: Messages;
}

function resolve(choice: LocaleChoice): I18nState {
  const locale = choice === "system" ? systemLocale() : choice;
  return { choice, locale, messages: CATALOG[locale] };
}

function markDocument(locale: Locale): void {
  if (typeof document !== "undefined") {
    document.documentElement.lang = locale;
  }
}

const store = createStore<I18nState>(() => resolve(readChoice()));
markDocument(store.getState().locale);

/** Switches the language now and remembers it. */
export function setLocaleChoice(choice: LocaleChoice): void {
  try {
    localStorage.setItem(STORAGE_KEY, choice);
  } catch {
    // Not remembered, but still switched for this run.
  }
  const next = resolve(choice);
  markDocument(next.locale);
  store.setState(next);
}

/** The current messages, re-rendering when the language changes. */
export function useT(): Messages {
  return useStore(store, (state) => state.messages);
}

export function useLocale(): { choice: LocaleChoice; locale: Locale } {
  return useStore(
    store,
    useShallow((state) => ({ choice: state.choice, locale: state.locale })),
  );
}

/** The current messages, for code outside React. */
export function t(): Messages {
  return store.getState().messages;
}

export function currentLocale(): Locale {
  return store.getState().locale;
}
