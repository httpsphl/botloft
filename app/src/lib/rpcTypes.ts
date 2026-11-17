// The types of the connection to botloftd (spec 11): methods, params and
// results from the generated protocol, the errors of either side, the
// connection's states and the part of a WebSocket it uses.

import type { RpcMethods, RpcNotifications } from "./protocol.gen";

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
  /** Why a validation failed, when the app words it itself (spec 20.8). */
  readonly reason: string | undefined;

  constructor(code: number, message: string, reason?: string) {
    super(message);
    this.name = "RpcError";
    this.code = code;
    this.reason = reason;
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
