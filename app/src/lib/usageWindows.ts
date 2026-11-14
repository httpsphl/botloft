// How a window of the plan reads in the usage dialog and the island (spec
// 8.13): by its name, or as a model's own week.

import type { Messages } from "../i18n";
import type { UsageWindow } from "./protocol.gen";

/** "This week", "5-hour window", "This week, Fable". */
export function windowName(window: UsageWindow, words: Messages["account"]["usage"]): string {
  if (window.model) {
    return words.modelWeek(window.model);
  }
  return words.windows[window.name] ?? window.name.replaceAll("_", " ");
}

/** A key that tells two windows of one name apart by their model. */
export function windowKey(window: UsageWindow): string {
  return `${window.name}/${window.model ?? ""}`;
}
