// Light and dark themes (spec 15.3): follows Windows unless the owner picks
// one. The choice is a per-machine convenience kept in localStorage.

import { useEffect, useSyncExternalStore } from "react";

export type ThemeChoice = "system" | "light" | "dark";

const KEY = "botloft.theme";
const listeners = new Set<() => void>();

function stored(): ThemeChoice {
  try {
    const value = localStorage.getItem(KEY);
    return value === "light" || value === "dark" ? value : "system";
  } catch {
    return "system";
  }
}

let choice: ThemeChoice = stored();

function systemDark(): boolean {
  return typeof matchMedia === "function" && matchMedia("(prefers-color-scheme: dark)").matches;
}

export function resolvedTheme(which: ThemeChoice = choice): "light" | "dark" {
  if (which === "system") {
    return systemDark() ? "dark" : "light";
  }
  return which;
}

function apply(): void {
  document.documentElement.dataset.theme = resolvedTheme();
  for (const listener of listeners) {
    listener();
  }
}

export function setTheme(next: ThemeChoice): void {
  choice = next;
  try {
    localStorage.setItem(KEY, next);
  } catch {
    // Private storage off: the choice lasts for this window.
  }
  apply();
}

/** Applies the theme and follows Windows while the choice is "system". */
export function useThemeRoot(): void {
  useEffect(() => {
    apply();
    if (typeof matchMedia !== "function") {
      return;
    }
    const media = matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => choice === "system" && apply();
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function useTheme(): { choice: ThemeChoice; resolved: "light" | "dark" } {
  const current = useSyncExternalStore(subscribe, () => choice);
  const resolved = useSyncExternalStore(subscribe, () => resolvedTheme());
  return { choice: current, resolved };
}
