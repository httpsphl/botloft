// The phone's connection (spec 28.4): one WebSocket to the relay, the sealed
// messages both ways, and what the screen shows, which is in memory only.

import type {
  ApprovalCard,
  ApprovalStatus,
  FromPhone,
  QuestionCard,
  QuestionStatus,
  ToPhone,
} from "../lib/protocol.gen";
import { open, SealError, seal } from "./crypto";
import type { Session, SessionStore } from "./store";

export type Ended = Exclude<ApprovalStatus, "pending"> | Exclude<QuestionStatus, "open">;

export interface PhoneState {
  /** `revoked`: the server or the computer cut this phone off. `left`: the owner left from here. */
  session: "ok" | "revoked" | "left";
  link: "connecting" | "online" | "offline";
  /** Whether the computer is on, as the relay says. */
  computer: "unknown" | "online" | "offline";
  /** The first list from the computer arrived. */
  loaded: boolean;
  approvals: ApprovalCard[];
  questions: QuestionCard[];
  /** Answers on their way, by the id of what they answer. */
  sending: string[];
  /** How the last things that closed ended, for a word on screen. */
  ended: Record<string, Ended>;
  name: string;
}

/** What the screens use; the real client and the fake both are one. */
export interface PhoneApi {
  getState(): PhoneState;
  subscribe(listener: () => void): () => void;
  /** False when there is no connection to send it through. */
  answerApproval(id: string, allow: boolean, note?: string): Promise<boolean>;
  answerQuestion(id: string, answer: string): Promise<boolean>;
  dismissQuestion(id: string): Promise<boolean>;
  /** Leaves from this side: the computer is told, and the keys go. */
  disconnect(): Promise<void>;
}

export interface Deps {
  /** The server's address, such as `https://cloud.example.org`. */
  origin: string;
  webSocket: typeof WebSocket;
  fetch: typeof fetch;
  store: SessionStore;
  /** How long to wait before trying again: 1 s, doubling to 30 s. */
  retry?: { min: number; max: number };
  /** How long an answer may be on its way before the button comes back. */
  answerWait?: number;
}

const text = new TextEncoder();
const decode = new TextDecoder();
const OPEN = 1;

function oldestFirst<T extends { createdAt: number }>(cards: T[]): T[] {
  return [...cards].sort((a, b) => a.createdAt - b.createdAt);
}

export class PhoneClient implements PhoneApi {
  private state: PhoneState;
  private readonly listeners = new Set<() => void>();
  private socket: WebSocket | null = null;
  private stopped = false;
  private delay: number;
  private timer: ReturnType<typeof setTimeout> | null = null;
  /** Frames and sends go one at a time, in order: the counters depend on it. */
  private inbox: Promise<void> = Promise.resolve();
  private outbox: Promise<unknown> = Promise.resolve();
  private saving: Promise<void> = Promise.resolve();

  constructor(
    private session: Session,
    private readonly deps: Deps,
  ) {
    this.delay = deps.retry?.min ?? 1000;
    this.state = {
      session: "ok",
      link: "connecting",
      computer: "unknown",
      loaded: false,
      approvals: [],
      questions: [],
      sending: [],
      ended: {},
      name: session.name,
    };
  }

  getState = (): PhoneState => this.state;

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  private set(change: Partial<PhoneState>): void {
    this.state = { ...this.state, ...change };
    for (const listener of this.listeners) {
      listener();
    }
  }

  start(): void {
    this.stopped = false;
    this.connect();
  }

  stop(): void {
    this.stopped = true;
    if (this.timer) {
      clearTimeout(this.timer);
    }
    this.socket?.close();
    this.socket = null;
  }

  private url(): string {
    return `${this.deps.origin.replace(/^http/, "ws")}/v1/relay`;
  }

  private connect(): void {
    this.set({ link: "connecting" });
    const socket = new this.deps.webSocket(this.url());
    this.socket = socket;
    socket.onopen = () => socket.send(JSON.stringify({ t: "hello", token: this.session.token }));
    socket.onmessage = (event) => {
      const frame = String(event.data);
      this.inbox = this.inbox.then(() => this.onFrame(frame)).catch(() => {});
    };
    socket.onclose = () => {
      if (this.socket !== socket) {
        return;
      }
      this.socket = null;
      if (this.state.session !== "ok" || this.stopped) {
        return;
      }
      this.set({ link: "offline", computer: "unknown" });
      this.timer = setTimeout(() => this.connect(), this.delay);
      this.delay = Math.min(this.delay * 2, this.deps.retry?.max ?? 30000);
    };
  }

  private async onFrame(raw: string): Promise<void> {
    let frame: Record<string, unknown>;
    try {
      frame = JSON.parse(raw);
    } catch {
      return;
    }
    switch (frame.t) {
      case "ready":
        this.delay = this.deps.retry?.min ?? 1000;
        this.set({ link: "online", computer: frame.online === true ? "online" : "offline" });
        await this.send({ t: "sync" });
        break;
      case "presence":
        if (frame.device === this.session.peer) {
          const online = frame.online === true;
          this.set({ computer: online ? "online" : "offline" });
          if (online) {
            await this.send({ t: "sync" });
          }
        }
        break;
      case "msg":
        await this.onMessage(frame);
        break;
      case "error":
        if (frame.reason === "unauthorized") {
          this.cut("revoked");
        }
        break;
      default:
    }
  }

  private async onMessage(frame: Record<string, unknown>): Promise<void> {
    if (typeof frame.body !== "string") {
      return;
    }
    let opened: { seq: number; plain: Uint8Array };
    try {
      opened = await open(
        this.session.keys.c2p,
        "computerToPhone",
        this.session.device,
        frame.body,
      );
    } catch (failure) {
      if (failure instanceof SealError) {
        return;
      }
      throw failure;
    }
    // The same message again, or an old one sent back.
    if (opened.seq !== frame.seq || opened.seq <= this.session.received) {
      return;
    }
    this.session = { ...this.session, received: opened.seq };
    await this.persist();
    let message: ToPhone;
    try {
      message = JSON.parse(decode.decode(opened.plain));
    } catch {
      return;
    }
    this.apply(message);
  }

  private apply(message: ToPhone): void {
    const { state } = this;
    switch (message.t) {
      case "snapshot":
        this.set({
          loaded: true,
          approvals: oldestFirst(message.approvals),
          questions: oldestFirst(message.questions),
        });
        break;
      case "approval.open":
        this.set({
          approvals: oldestFirst([
            ...state.approvals.filter((card) => card.approvalId !== message.card.approvalId),
            message.card,
          ]),
          sending: state.sending.filter((id) => id !== message.card.approvalId),
        });
        break;
      case "approval.closed":
        if (message.status !== "pending") {
          this.close(message.approvalId, message.status);
        }
        break;
      case "question.open":
        this.set({
          questions: oldestFirst([
            ...state.questions.filter((card) => card.questionId !== message.card.questionId),
            message.card,
          ]),
        });
        break;
      case "question.closed":
        if (message.status !== "open") {
          this.close(message.questionId, message.status);
        }
        break;
      default:
    }
  }

  private close(id: string, status: Ended): void {
    const { state } = this;
    const ended = { ...state.ended, [id]: status };
    for (const old of Object.keys(ended).slice(0, -20)) {
      delete ended[old];
    }
    this.set({
      approvals: state.approvals.filter((card) => card.approvalId !== id),
      questions: state.questions.filter((card) => card.questionId !== id),
      sending: state.sending.filter((sending) => sending !== id),
      ended,
    });
  }

  private persist(): Promise<void> {
    const session = this.session;
    this.saving = this.saving.then(() => this.deps.store.save(session)).catch(() => {});
    return this.saving;
  }

  /** Seals and sends one message, after the ones before it. False if there is no connection. */
  private send(message: FromPhone): Promise<boolean> {
    const run = async (): Promise<boolean> => {
      const socket = this.socket;
      if (!socket || socket.readyState !== OPEN || this.state.link !== "online") {
        return false;
      }
      const seq = this.session.sent + 1;
      this.session = { ...this.session, sent: seq };
      // Saved before the message goes, so a counter is never used twice.
      await this.persist();
      const body = await seal(
        this.session.keys.p2c,
        "phoneToComputer",
        this.session.device,
        seq,
        text.encode(JSON.stringify(message)),
      );
      socket.send(JSON.stringify({ t: "msg", seq, body }));
      return true;
    };
    const sent = this.outbox.then(run, run);
    this.outbox = sent.catch(() => false);
    return sent;
  }

  private async answer(id: string, message: FromPhone): Promise<boolean> {
    this.set({ sending: [...this.state.sending.filter((other) => other !== id), id] });
    const sent = await this.send(message).catch(() => false);
    if (!sent) {
      this.set({ sending: this.state.sending.filter((other) => other !== id) });
      return false;
    }
    // If nothing comes back, the button returns and the owner may try again.
    setTimeout(
      () => this.set({ sending: this.state.sending.filter((other) => other !== id) }),
      this.deps.answerWait ?? 15000,
    );
    return true;
  }

  answerApproval = (id: string, allow: boolean, note?: string): Promise<boolean> => {
    const trimmed = note?.trim();
    return this.answer(id, {
      t: "approval.answer",
      approvalId: id,
      allow,
      ...(trimmed ? { note: trimmed } : {}),
    });
  };

  answerQuestion = (id: string, answer: string): Promise<boolean> =>
    this.answer(id, { t: "question.answer", questionId: id, answer });

  dismissQuestion = (id: string): Promise<boolean> =>
    this.answer(id, { t: "question.dismiss", questionId: id });

  private cut(session: "revoked" | "left"): void {
    this.stop();
    this.set({ session, link: "offline", computer: "unknown", approvals: [], questions: [] });
  }

  disconnect = async (): Promise<void> => {
    try {
      await this.deps.fetch(`${this.deps.origin}/v1/logout`, {
        method: "POST",
        headers: { Authorization: `Bearer ${this.session.token}` },
      });
    } catch {
      // The keys go either way; the computer drops the phone at its next look.
    }
    await this.forget("left");
  };

  /** The phone was cut off from the other side: nothing here can work any more. */
  forget = async (session: "revoked" | "left" = "revoked"): Promise<void> => {
    this.cut(session);
    await this.deps.store.clear();
  };
}
