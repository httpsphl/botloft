// A phone connection for the tests and the preview: the screens read its
// state and call its methods, and nothing leaves the page.

import type { ApprovalCard, FromPhone, QuestionCard, ToPhone } from "../lib/protocol.gen";
import type { PhoneApi, PhoneState } from "./client";
import type { NoticeState } from "./notices";
import type { LockApi } from "./PinSettings";
import { noChats, type SeenStore, Talk } from "./talk";

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
    ...noChats,
  };
  /** What the screens asked of the computer, in order. */
  readonly asked: FromPhone[] = [];
  private readonly talk: Talk;

  constructor(seen?: SeenStore) {
    this.talk = new Talk(
      () => this.state,
      (change) => this.set(change),
      async (message) => {
        this.asked.push(message);
        return this.sendable;
      },
      seen,
    );
  }
  private readonly listeners = new Set<() => void>();
  /** How notices stand; the buttons change it as the browser would. */
  notices: NoticeState = "off";
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
  noticeState = async (): Promise<NoticeState> => this.notices;
  turnOnNotices = async (): Promise<NoticeState> => {
    this.calls.push({ method: "turnOnNotices", args: [] });
    this.notices = this.notices === "off" ? "on" : this.notices;
    return this.notices;
  };
  turnOffNotices = async (): Promise<NoticeState> => {
    this.calls.push({ method: "turnOffNotices", args: [] });
    this.notices = this.notices === "on" ? "off" : this.notices;
    return this.notices;
  };
  /** Plays a message the computer sends about conversations. */
  receive(message: ToPhone): void {
    this.talk.handle(message);
  }

  openChat = (botId: string) => this.talk.open(botId);
  closeChat = () => this.talk.leave();
  olderItems = () => this.talk.older();
  write = (text: string) => this.talk.write(text);
  disconnect = async () => {
    this.calls.push({ method: "disconnect", args: [] });
    this.set({ session: "left" });
  };
}

/** The page's lock, for the tests: it keeps the PIN it was given. */
export class FakeLock implements LockApi {
  state_: "off" | "on" | "old" = "off";
  pin: string | null = null;
  dismissed = false;
  readonly calls: string[] = [];

  state = async () => this.state_;
  setPin = async (pin: string) => {
    this.calls.push("setPin");
    this.pin = pin;
    this.state_ = "on";
  };
  removePin = async (pin: string) => {
    this.calls.push("removePin");
    if (pin !== this.pin) {
      return false;
    }
    this.pin = null;
    this.state_ = "off";
    return true;
  };
  nudgeDismissed = () => this.dismissed;
  dismissNudge = () => {
    this.calls.push("dismissNudge");
    this.dismissed = true;
  };
}
