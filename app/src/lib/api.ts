// The contract the UI uses to talk to the daemon (spec 15.1). Components
// depend on this interface only: `client.ts` implements it over the
// WebSocket and `fake.ts` in memory for tests.

import type { ConnectionState, Method, ParamsArg, Result, ServerEvent } from "./rpc";

export type { ConnectionState, Method, Params, Result, ServerEvent } from "./rpc";
export { ClientErrorCode, RpcError } from "./rpc";

export interface BotloftApi {
  /** Calls a daemon method; rejects with `RpcError`. */
  call<M extends Method>(method: M, ...params: ParamsArg<M>): Promise<Result<M>>;
  /** Server notifications, in order. Returns the unsubscribe function. */
  subscribe(listener: (event: ServerEvent) => void): () => void;
  connection(): ConnectionState;
  onConnection(listener: (state: ConnectionState) => void): () => void;
}

/** A readable sentence for a failed call. */
export function errorText(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}
