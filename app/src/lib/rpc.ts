// JSON-RPC 2.0 over WebSocket to botloftd (spec 11): `session.hello`
// first, requests queued while disconnected, reconnection with backoff, and
// a ping when the connection goes quiet, so one the network left half open
// is noticed and replaced.

import { t } from "../i18n";
import { type ClientInfo, type HelloResult, PROTOCOL_VERSION, RpcErrorCode } from "./protocol.gen";
import { readFrame } from "./rpcFrame";
import {
  ClientErrorCode,
  type ConnectionState,
  type Method,
  type ParamsArg,
  type Result,
  RpcError,
  type ServerEvent,
  type SocketLike,
} from "./rpcTypes";

export * from "./rpcTypes";

/**
 * Requests that may take minutes: reading the owner's whole Claude Code
 * history with a cold disk took over 30 s (spec 36.7).
 */
const SLOW_MS: Partial<Record<Method, number>> = {
  "imports.scan": 10 * 60_000,
  "imports.create": 10 * 60_000,
};

export interface RpcOptions {
  url: string;
  token: string;
  client: ClientInfo;
  socket?: (url: string) => SocketLike;
  retryInitialMs?: number;
  retryMaxMs?: number;
  requestTimeoutMs?: number;
  /** How long the hello may go unanswered before the socket is replaced. */
  helloTimeoutMs?: number;
  /**
   * Quiet this long, the connection sends `session.ping`; with no answer in
   * as long again, the socket is replaced.
   */
  heartbeatMs?: number;
}

interface Pending {
  method: string;
  params: unknown;
  resolve: (value: unknown) => void;
  reject: (error: RpcError) => void;
  timer: ReturnType<typeof setTimeout> | undefined;
}

export class RpcConnection {
  private readonly options: Required<RpcOptions>;
  private socket: SocketLike | null = null;
  private nextId = 1;
  private retryMs: number;
  private retryTimer: ReturnType<typeof setTimeout> | undefined;
  private beatTimer: ReturnType<typeof setInterval> | undefined;
  /** When anything last came from the daemon. */
  private lastHeard = 0;
  private pinging = false;
  /** Waiting for the connection to open. */
  private readonly queue: Pending[] = [];
  /** Sent, waiting for the answer. */
  private readonly inflight = new Map<number, Pending>();
  private readonly notificationListeners = new Set<(event: ServerEvent) => void>();
  private readonly stateListeners = new Set<(state: ConnectionState) => void>();
  state: ConnectionState = { kind: "closed" };

  constructor(options: RpcOptions) {
    this.options = {
      socket: (url) => new WebSocket(url) as unknown as SocketLike,
      retryInitialMs: 500,
      retryMaxMs: 5000,
      requestTimeoutMs: 30_000,
      helloTimeoutMs: 10_000,
      heartbeatMs: 15_000,
      ...options,
    };
    this.retryMs = this.options.retryInitialMs;
  }

  start(): void {
    if (this.state.kind === "closed" || this.state.kind === "failed") {
      this.open();
    }
  }

  /** Stops for good: pending requests fail and nothing reconnects. */
  close(): void {
    clearTimeout(this.retryTimer);
    this.setState({ kind: "closed" });
    this.dropSocket();
    this.failAll(new RpcError(ClientErrorCode.closed, t().common.errors.closed));
  }

  request<M extends Method>(method: M, ...params: ParamsArg<M>): Promise<Result<M>> {
    return new Promise<Result<M>>((resolve, reject) => {
      if (this.state.kind === "closed" || this.state.kind === "failed") {
        reject(new RpcError(ClientErrorCode.closed, t().common.errors.notConnected));
        return;
      }
      const pending: Pending = {
        method,
        params: params[0],
        resolve: resolve as (value: unknown) => void,
        reject,
        timer: undefined,
      };
      if (this.state.kind === "open") {
        this.send(pending);
      } else {
        this.queue.push(pending);
      }
    });
  }

  onNotification(listener: (event: ServerEvent) => void): () => void {
    this.notificationListeners.add(listener);
    return () => this.notificationListeners.delete(listener);
  }

  onState(listener: (state: ConnectionState) => void): () => void {
    this.stateListeners.add(listener);
    return () => this.stateListeners.delete(listener);
  }

  private open(): void {
    this.setState({ kind: "connecting" });
    const socket = this.options.socket(this.options.url);
    this.socket = socket;
    socket.onopen = () => this.hello(socket);
    socket.onmessage = (event) => {
      if (typeof event.data === "string") {
        this.receive(event.data);
      }
    };
    socket.onclose = () => this.lost(socket);
    socket.onerror = () => this.lost(socket);
  }

  private hello(socket: SocketLike): void {
    const id = this.nextId++;
    this.lastHeard = Date.now();
    const done = (result: HelloResult) => {
      this.retryMs = this.options.retryInitialMs;
      this.setState({ kind: "open", daemonVersion: result.daemonVersion });
      clearInterval(this.beatTimer);
      this.beatTimer = setInterval(() => this.beat(socket), this.options.heartbeatMs);
      for (const pending of this.queue.splice(0)) {
        this.send(pending);
      }
    };
    const refused = (error: RpcError) => {
      // A wrong token or protocol will not fix itself by retrying.
      const fatal =
        error.code === RpcErrorCode.notAuthenticated || error.code === RpcErrorCode.validation;
      if (fatal) {
        this.setState({ kind: "failed", reason: error.message });
        this.dropSocket();
        this.failAll(error);
      } else {
        // Anything else may pass: try again, rather than wait for nothing.
        this.lost(socket);
      }
    };
    this.inflight.set(id, {
      method: "session.hello",
      params: undefined,
      resolve: (value) => done(value as HelloResult),
      reject: refused,
      timer: setTimeout(() => {
        this.inflight.delete(id);
        this.lost(socket);
      }, this.options.helloTimeoutMs),
    });
    const params = {
      token: this.options.token,
      client: this.options.client,
      protocol: PROTOCOL_VERSION,
    };
    socket.send(JSON.stringify({ jsonrpc: "2.0", id, method: "session.hello", params }));
  }

  /**
   * Quiet for a beat: asks the daemon whether it is there, which it answers
   * at once (spec 11). No answer within the next beat, the socket is taken
   * as lost and replaced.
   */
  private beat(socket: SocketLike): void {
    if (
      socket !== this.socket ||
      this.pinging ||
      Date.now() - this.lastHeard < this.options.heartbeatMs
    ) {
      return;
    }
    const id = this.nextId++;
    this.pinging = true;
    const answered = () => {
      this.pinging = false;
    };
    this.inflight.set(id, {
      method: "session.ping",
      params: undefined,
      resolve: answered,
      reject: answered,
      timer: setTimeout(() => {
        this.inflight.delete(id);
        this.pinging = false;
        this.lost(socket);
      }, this.options.heartbeatMs),
    });
    socket.send(JSON.stringify({ jsonrpc: "2.0", id, method: "session.ping" }));
  }

  private send(pending: Pending): void {
    const id = this.nextId++;
    pending.timer = setTimeout(
      () => {
        this.inflight.delete(id);
        pending.reject(
          new RpcError(ClientErrorCode.timeout, t().common.errors.timedOut(pending.method)),
        );
      },
      SLOW_MS[pending.method as Method] ?? this.options.requestTimeoutMs,
    );
    this.inflight.set(id, pending);
    const frame: Record<string, unknown> = { jsonrpc: "2.0", id, method: pending.method };
    if (pending.params !== undefined) {
      frame.params = pending.params;
    }
    this.socket?.send(JSON.stringify(frame));
  }

  private receive(text: string): void {
    this.lastHeard = Date.now();
    const frame = readFrame(text);
    if (frame?.kind === "answer") {
      const pending = this.inflight.get(frame.id);
      if (!pending) {
        return;
      }
      this.inflight.delete(frame.id);
      clearTimeout(pending.timer);
      if ("error" in frame.outcome) {
        pending.reject(frame.outcome.error);
      } else {
        pending.resolve(frame.outcome.result);
      }
    } else if (frame?.kind === "event") {
      for (const listener of this.notificationListeners) {
        listener(frame.event);
      }
    }
  }

  private lost(socket: SocketLike): void {
    if (socket !== this.socket) {
      return;
    }
    this.dropSocket();
    // Requests already sent may or may not have run; the caller decides.
    for (const [id, pending] of this.inflight) {
      this.inflight.delete(id);
      clearTimeout(pending.timer);
      pending.reject(
        new RpcError(ClientErrorCode.connectionLost, t().common.errors.connectionLost),
      );
    }
    if (this.state.kind === "failed" || this.state.kind === "closed") {
      return;
    }
    const delay = this.retryMs;
    this.retryMs = Math.min(this.retryMs * 2, this.options.retryMaxMs);
    this.setState({ kind: "waiting", retryAt: Date.now() + delay });
    this.retryTimer = setTimeout(() => this.open(), delay);
  }

  private dropSocket(): void {
    clearInterval(this.beatTimer);
    this.pinging = false;
    const socket = this.socket;
    this.socket = null;
    if (socket) {
      socket.onopen = null;
      socket.onmessage = null;
      socket.onclose = null;
      socket.onerror = null;
      socket.close();
    }
  }

  private failAll(error: RpcError): void {
    for (const pending of [...this.queue.splice(0), ...this.inflight.values()]) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }
    this.inflight.clear();
  }

  private setState(state: ConnectionState): void {
    this.state = state;
    for (const listener of this.stateListeners) {
      listener(state);
    }
  }
}
