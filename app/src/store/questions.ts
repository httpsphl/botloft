// Questions the bots asked the owner (spec 23.5): the open ones, loaded
// with the rest and kept current by `question.changed`.

import type { Question, QuestionId } from "../lib/protocol.gen";
import type { AppState } from "./app";

/** Keeps an open question; one answered or dismissed leaves. */
export function withQuestion(state: AppState, question: Question): Partial<AppState> {
  const { [question.id]: _, ...rest } = state.questions;
  return {
    questions: question.status === "open" ? { ...rest, [question.id]: question } : rest,
  };
}

export function questionsById(list: Question[]): Record<QuestionId, Question> {
  return Object.fromEntries(list.map((question) => [question.id, question]));
}

let last: { questions: AppState["questions"]; bots: AppState["bots"]; list: Question[] } | null =
  null;

/**
 * Open questions of the bots the app shows, newest first. The same array
 * while nothing it depends on changed, for selectors.
 */
export function openQuestions(state: AppState): Question[] {
  if (last && last.questions === state.questions && last.bots === state.bots) {
    return last.list;
  }
  const list = Object.values(state.questions)
    .filter((question) => state.bots[question.botId] !== undefined)
    .sort((a, b) => b.createdAt - a.createdAt || b.id.localeCompare(a.id));
  last = { questions: state.questions, bots: state.bots, list };
  return list;
}
