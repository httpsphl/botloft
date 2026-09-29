// JSON-RPC 2.0 over WebSocket to botloftd (spec 11): `session.hello`
// first, requests queued while disconnected, and reconnection with backoff.

import { t } from "../i18n";
import {
  type ClientInfo,
  type HelloResult,
  PROTOCOL_VERSION,
  RpcErrorCode,
  type RpcMethods,
  type RpcNotifications,
} from "./protocol.gen";

export type Method = keyof RpcMethods;
export type Params<M extends Method> = RpcMethods[M]["params"];
export type Result<M extends Method> = RpcMethods[M]["result"];
/** No argument for methods without params. */
export type ParamsArg<M extends Method> = Params<M> extends undefined ? [] : [Params<M>];
export type NotificationName = keyof RpcNotifications;
export type ServerEvent = {
  [N in NotificationName]: { name: N; params: RpcNotifications[N] };
}[NotificationName];

/** Codes for failures that happen on this side of the socket. */
export const ClientErrorCode = {
  connectionLost: -1,
  timeout: -2,
  closed: -3,
} as const;

export class RpcError extends Error {
  readonly code: number;

  constructor(code: number, message: string) {
    super(message);
    this.name = "RpcError";
    this.code = code;
  }
}

export type ConnectionState =
  | { kind: "connecting" }
  | { kind: "open"; daemonVersion: string }
  /** Lost the daemon; trying again at `retryAt` (ms). */
  | { kind: "waiting"; retryAt: number }
  /** The daemon refused this client (token or protocol); no retries. */
  | { kind: "failed"; reason: string }
  | { kind: "closed" };

/** The part of `WebSocket` the connection uses. */
export interface SocketLike {
  onopen: ((event: unknown) => void) | null;
  onmessage: ((event: { data: unknown }) => void) | null;
  onclose: ((event: unknown) => void) | null;
  onerror: ((event: unknown) => void) | null;
  send(data: string): void;
  close(): void;
}

export interface RpcOptions {
  url: string;
  token: string;
  client: ClientInfo;
  socket?: (url: string) => SocketLike;
  retryInitialMs?: number;
  retryMaxMs?: number;
  requestTimeoutMs?: number;
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
    const done = (result: HelloResult) => {
      this.retryMs = this.options.retryInitialMs;
      this.setState({ kind: "open", daemonVersion: result.daemonVersion });
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
      }
    };
    this.inflight.set(id, {
      method: "session.hello",
      params: undefined,
      resolve: (value) => done(value as HelloResult),
      reject: refused,
      timer: undefined,
    });
    const params = {
      token: this.options.token,
      client: this.options.client,
      protocol: PROTOCOL_VERSION,
    };
    socket.send(JSON.stringify({ jsonrpc: "2.0", id, method: "session.hello", params }));
  }

  private send(pending: Pending): void {
    const id = this.nextId++;
    pending.timer = setTimeout(() => {
      this.inflight.delete(id);
      pending.reject(
        new RpcError(ClientErrorCode.timeout, t().common.errors.timedOut(pending.method)),
      );
    }, this.options.requestTimeoutMs);
    this.inflight.set(id, pending);
    const frame: Record<string, unknown> = { jsonrpc: "2.0", id, method: pending.method };
    if (pending.params !== undefined) {
      frame.params = pending.params;
    }
    this.socket?.send(JSON.stringify(frame));
  }

  private receive(text: string): void {
    let frame: {
      id?: unknown;
      method?: unknown;
      params?: unknown;
      result?: unknown;
      error?: { code?: unknown; message?: unknown };
    };
    try {
      frame = JSON.parse(text);
    } catch {
      return;
    }
    if (typeof frame.id === "number") {
      const pending = this.inflight.get(frame.id);
      if (!pending) {
        return;
      }
      this.inflight.delete(frame.id);
      clearTimeout(pending.timer);
      if (frame.error) {
        const code = typeof frame.error.code === "number" ? frame.error.code : 0;
        const message = typeof frame.error.message === "string" ? frame.error.message : "error";
        pending.reject(new RpcError(code, message));
      } else {
        pending.resolve(frame.result ?? null);
      }
    } else if (typeof frame.method === "string") {
      const event = { name: frame.method, params: frame.params } as ServerEvent;
      for (const listener of this.notificationListeners) {
        listener(event);
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
