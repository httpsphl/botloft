// The app's own SSH tunnel to Botloft on a server (spec 42.6): what the
// app's Tauri side needs to open it, the port it listens on now, and what
// went wrong when it could not open it.

import type { TunnelSpec } from "../lib/host";
import type { ServerProfile } from "./server";

/** What went wrong opening the tunnel, from the Tauri side's own words. */
export type TunnelReason = "unreachable" | "refused" | "host_key" | "no_ssh" | "failed";

export interface TunnelProblem {
  reason: TunnelReason;
  /** What `ssh` said, for "Details". */
  detail: string;
}

const NAMED: TunnelReason[] = ["unreachable", "refused", "host_key", "no_ssh"];

/** The problem in an error the Tauri side gave (`reason: what ssh said`). */
export function tunnelProblem(error: unknown): TunnelProblem {
  const text = error instanceof Error ? error.message : String(error);
  const colon = text.indexOf(": ");
  const head = colon < 0 ? "" : text.slice(0, colon);
  const named = NAMED.find((reason) => reason === head);
  return named
    ? { reason: named, detail: text.slice(colon + 2) }
    : { reason: "failed", detail: text };
}

/** Whether the problem may pass by itself, so the app tries again. */
export function mayPass(problem: TunnelProblem): boolean {
  return problem.reason === "unreachable";
}

/** What opens the tunnel to `chosen`, when its key says how. */
export function tunnelSpec(chosen: ServerProfile): TunnelSpec | null {
  const ssh = chosen.ssh;
  if (!ssh) {
    return null;
  }
  return {
    host: ssh.host,
    port: ssh.port,
    user: ssh.user,
    hostKey: ssh.hostKey,
    key: ssh.key,
    remotePort: chosen.port,
    localPort: chosen.tunnel,
  };
}

/** The port the app's own tunnel listens on now, when one runs. */
let openPort: number | null = null;

/** Where the tunnel to `chosen` starts on this computer. */
export function tunnelPort(chosen: ServerProfile): number {
  return chosen.ssh && openPort !== null ? openPort : chosen.tunnel;
}

/** The app's own tunnel listens on `port` now; null once it is closed. */
export function setTunnelPort(port: number | null): void {
  openPort = port;
}
