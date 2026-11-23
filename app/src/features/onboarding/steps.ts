// Where the link to the daemon is (spec 15.1), on this computer or on a
// server (42.6): what the onboarding screen shows for each step.

import type { Client } from "../../lib/client";
import type { ServerProfile } from "../../shell/server";
import type { TunnelProblem } from "../../shell/tunnel";

export type LinkStep =
  | { step: "checking" }
  /** Not running, and installing it did not help. */
  | { step: "stopped"; port: number; home: string; error: string | null }
  /**
   * Installing, updating to this app's version or restarting the daemon, or
   * starting it again after the owner closed Botloft with the bots.
   */
  | { step: "installing"; action: "install" | "update" | "restart" | "start" }
  /** Another program holds the daemon's port. */
  | { step: "foreign"; port: number }
  /** The daemon speaks another protocol version and is not older than the app. */
  | { step: "mismatch"; daemonProtocol: number; daemonVersion: string }
  /** The daemon is older than this app and updating it failed. */
  | { step: "outdated"; daemonVersion: string; error: string }
  | { step: "error"; message: string }
  /** The app opens its own tunnel to the server (spec 42.6). */
  | { step: "opening"; server: ServerProfile }
  /** `server` when it is one, not this computer (spec 42.6). */
  | { step: "connecting"; api: Client; server?: ServerProfile }
  | { step: "connected"; api: Client }
  /** The daemon refused the hello (token or protocol). */
  | { step: "refused"; reason: string; server?: ServerProfile }
  /**
   * Nothing answers where the tunnel to the server starts, or the app's own
   * tunnel did not open (`problem`). The app keeps trying when it may pass.
   */
  | { step: "unreachable"; server: ServerProfile; problem?: TunnelProblem };

/** A client to the daemon on this computer's `port`, signed in with `token`. */
export type Connect = (port: number, token: string) => Client;
