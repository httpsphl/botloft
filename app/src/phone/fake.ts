// A phone connection for the tests and the preview: the screens read its
// state and call its methods, and nothing leaves the page.

import type { ApprovalCard, QuestionCard } from "../lib/protocol.gen";
import type { PhoneApi, PhoneState } from "./client";

export class FakePhone implements PhoneApi {
  private state: PhoneState = {
    session: "ok",
    link: "online",
    computer: "online",
    loaded: true,
    approvals: [],
    questions: [],
    sending: [],
    ended: {},
    name: "Celular da Ana",
  };
  private readonly listeners = new Set<() => void>();
  /** Whether an answer can be sent, for a test of the failure. */
  sendable = true;
  readonly calls: { method: string; args: unknown[] }[] = [];

  getState = (): PhoneState => this.state;

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  /** Plays a change the computer or the connection makes. */
  set(change: Partial<PhoneState>): void {
    this.state = { ...this.state, ...change };
    for (const listener of this.listeners) {
      listener();
    }
  }

  add(approval?: ApprovalCard, question?: QuestionCard): void {
    this.set({
      approvals: approval ? [...this.state.approvals, approval] : this.state.approvals,
      questions: question ? [...this.state.questions, question] : this.state.questions,
    });
  }

  private async answer(method: string, ...args: unknown[]): Promise<boolean> {
    this.calls.push({ method, args });
    return this.sendable;
  }

  answerApproval = (id: string, allow: boolean, note?: string) =>
    this.answer("answerApproval", id, allow, note);
  answerQuestion = (id: string, text: string) => this.answer("answerQuestion", id, text);
  dismissQuestion = (id: string) => this.answer("dismissQuestion", id);
  disconnect = async () => {
    this.calls.push({ method: "disconnect", args: [] });
    this.set({ session: "left" });
  };
}
