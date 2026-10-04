// A lesson as words (spec 21.13): the steps the owner showed, numbered, in
// the owner's language, for a routine's request or a message to the bot.
// What was typed was never kept, so the words say where, not what.

import type { Messages } from "../../i18n/en";
import type { LessonStep } from "../../lib/protocol.gen";

type Words = Messages["browser"]["lesson"];

export function stepText(step: LessonStep, w: Words): string {
  switch (step.kind) {
    case "open":
      return w.step.open(step.label);
    case "click":
      return w.step.click(step.label);
    case "type":
      return step.secret ? w.step.secret(step.label) : w.step.type(step.label);
    case "press":
      return w.step.press(step.label);
  }
}

/** The whole lesson: what it is, each step, and a word on what was typed. */
export function lessonText(name: string, steps: LessonStep[], w: Words): string {
  const lines = steps.map((step, index) => `${index + 1}. ${stepText(step, w)}`);
  const typed = steps.some((step) => step.kind === "type" && !step.secret);
  return [w.intro(name.trim()), ...lines, ...(typed ? ["", w.typedNote] : [])].join("\n");
}
