// Getting from "app opened" to "connected to the daemon": check that it
// runs, offer to start it, read the owner token and say hello (spec 15.1).

import { createStore, type StoreApi } from "zustand/vanilla";
import { errorText } from "../../lib/api";
import type { Client } from "../../lib/client";
import type { DaemonStatus, Host } from "../../lib/host";
import { PROTOCOL_VERSION } from "../../lib/protocol.gen";

export type LinkStep =
  | { step: "checking" }
  | { step: "stopped"; port: number; home: string; error: string | null }
  | { step: "starting" }
  /** Another program holds the daemon's port. */
  | { step: "foreign"; port: number }
  /** The daemon speaks another protocol version. */
  | { step: "mismatch"; daemonProtocol: number; daemonVersion: string }
  | { step: "error"; message: string }
  | { step: "connecting"; api: Client }
  | { step: "connected"; api: Client }
  /** The daemon refused the hello (token or protocol). */
  | { step: "refused"; reason: string };

export interface Link {
  current: LinkStep;
  /** Checks the daemon again from the start. */
  check(): Promise<void>;
  /** Starts the daemon, then connects. */
  start(): Promise<void>;
}

export type Connect = (port: number, token: string) => Client;

export function createLink(host: Host, connect: Connect): StoreApi<Link> {
  let api: Client | null = null;
  let generation = 0;

  const store = createStore<Link>()((set) => {
    const drop = () => {
      api?.close();
      api = null;
    };

    const attach = async (port: number, run: number) => {
      let token: string;
      try {
        token = await host.readOwnerToken();
      } catch (error) {
        if (run === generation) {
          set({ current: { step: "error", message: errorText(error) } });
        }
        return;
      }
      if (run !== generation) {
        return;
      }
      const client = connect(port, token);
      api = client;
      const follow = () => {
        const connection = client.connection();
        if (run !== generation) {
          return;
        }
        if (connection.kind === "open") {
          set({ current: { step: "connected", api: client } });
        } else if (connection.kind === "failed") {
          drop();
          set({ current: { step: "refused", reason: connection.reason } });
        }
      };
      set({ current: { step: "connecting", api: client } });
      client.onConnection(follow);
      follow();
    };

    const settle = async (status: DaemonStatus, run: number, error: string | null = null) => {
      if (run !== generation) {
        return;
      }
      switch (status.state) {
        case "stopped":
          set({ current: { step: "stopped", port: status.port, home: status.home, error } });
          return;
        case "foreign":
          set({ current: { step: "foreign", port: status.port } });
          return;
        case "running":
          if (status.protocol !== PROTOCOL_VERSION) {
            set({
              current: {
                step: "mismatch",
                daemonProtocol: status.protocol,
                daemonVersion: status.version,
              },
            });
            return;
          }
          await attach(status.port, run);
      }
    };

    return {
      current: { step: "checking" },
      check: async () => {
        generation += 1;
        const run = generation;
        drop();
        set({ current: { step: "checking" } });
        try {
          await settle(await host.daemonStatus(), run);
        } catch (error) {
          if (run === generation) {
            set({ current: { step: "error", message: errorText(error) } });
          }
        }
      },
      start: async () => {
        generation += 1;
        const run = generation;
        drop();
        set({ current: { step: "starting" } });
        try {
          const status = await host.startDaemon();
          const late =
            status.state === "stopped"
              ? "The daemon did not answer in time. Its log is in the logs folder of its data folder."
              : null;
          await settle(status, run, late);
        } catch (error) {
          if (run === generation) {
            set({ current: { step: "error", message: errorText(error) } });
          }
        }
      },
    };
  });
  return store;
}
