// One frame from botloftd (spec 11), read: the answer to a request, with
// its result or its error, or a notification. Anything else is nothing.

import { RpcError, type ServerEvent } from "./rpcTypes";

export type Frame =
  | { kind: "answer"; id: number; outcome: { result: unknown } | { error: RpcError } }
  | { kind: "event"; event: ServerEvent };

export function readFrame(text: string): Frame | null {
  let frame: {
    id?: unknown;
    method?: unknown;
    params?: unknown;
    result?: unknown;
    error?: { code?: unknown; message?: unknown; data?: { reason?: unknown } };
  };
  try {
    frame = JSON.parse(text);
  } catch {
    return null;
  }
  if (typeof frame.id === "number") {
    if (frame.error) {
      const code = typeof frame.error.code === "number" ? frame.error.code : 0;
      const message = typeof frame.error.message === "string" ? frame.error.message : "error";
      const reason = frame.error.data?.reason;
      const error = new RpcError(code, message, typeof reason === "string" ? reason : undefined);
      return { kind: "answer", id: frame.id, outcome: { error } };
    }
    return { kind: "answer", id: frame.id, outcome: { result: frame.result ?? null } };
  }
  if (typeof frame.method === "string") {
    return { kind: "event", event: { name: frame.method, params: frame.params } as ServerEvent };
  }
  return null;
}
