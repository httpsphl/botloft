// Getting from "app opened" to "connected to the daemon" without asking
// the owner anything: install it when it does not run, keep it at the
// version this app ships, read the owner token and say hello (spec 15.1).
// Each run installs or updates at most once, so a failure shows instead
// of looping.

import { createStore, type StoreApi } from "zustand/vanilla";
import { t } from "../../i18n";
import { errorText } from "../../lib/api";
import type { Client } from "../../lib/client";
import type { DaemonStatus, Host } from "../../lib/host";
import { PROTOCOL_VERSION } from "../../lib/protocol.gen";
import { whenClosed } from "../../shell/closing";

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
  | { step: "connecting"; api: Client }
  | { step: "connected"; api: Client }
  /** The daemon refused the hello (token or protocol). */
  | { step: "refused"; reason: string };

export interface Link {
  current: LinkStep;
  /** Checks the daemon again from the start, installing it if needed. */
  check(): Promise<void>;
  /** Installs the daemon as a scheduled task and starts it, then connects. */
  install(): Promise<void>;
  /** Restarts the daemon, then connects. */
  restart(): Promise<void>;
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

    /** Moves on from a status: installs a stopped daemon and updates an older one. */
    const settle = async (
      status: DaemonStatus,
      run: number,
      { error = null, mayChange = true }: { error?: string | null; mayChange?: boolean } = {},
    ): Promise<void> => {
      if (run !== generation) {
        return;
      }
      switch (status.state) {
        case "stopped":
          if (mayChange) {
            // Closing Botloft stopped the bots: opening it brings them back.
            const action = whenClosed() === "stop" ? "start" : "install";
            await change(run, action, () => host.installDaemon());
            return;
          }
          set({ current: { step: "stopped", port: status.port, home: status.home, error } });
          return;
        case "foreign":
          set({ current: { step: "foreign", port: status.port } });
          return;
        case "running":
          if (status.outdated) {
            if (!mayChange) {
              set({
                current: {
                  step: "outdated",
                  daemonVersion: status.version,
                  error: error ?? t().onboarding.outdated.stillOld(status.version),
                },
              });
              return;
            }
            await change(run, "update", () => host.installDaemon());
            return;
          }
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

    /** Installs, updates or restarts, then settles on what runs afterwards. */
    const change = async (
      run: number,
      action: "install" | "update" | "restart" | "start",
      work: () => Promise<DaemonStatus>,
    ) => {
      set({ current: { step: "installing", action } });
      let status: DaemonStatus;
      let error: string | null = null;
      try {
        status = await work();
        if (status.state === "stopped") {
          error = t().onboarding.late;
        }
      } catch (failure) {
        error = errorText(failure);
        try {
          status = await host.daemonStatus();
        } catch (again) {
          if (run === generation) {
            set({ current: { step: "error", message: errorText(again) } });
          }
          return;
        }
      }
      await settle(status, run, { error, mayChange: false });
    };

    const begin = () => {
      generation += 1;
      drop();
      return generation;
    };

    return {
      current: { step: "checking" },
      check: async () => {
        const run = begin();
        set({ current: { step: "checking" } });
        try {
          await settle(await host.daemonStatus(), run);
        } catch (error) {
          if (run === generation) {
            set({ current: { step: "error", message: errorText(error) } });
          }
        }
      },
      install: async () => {
        const run = begin();
        await change(run, "install", () => host.installDaemon());
      },
      restart: async () => {
        const run = begin();
        await change(run, "restart", () => host.restartDaemon());
      },
    };
  });
  return store;
}
