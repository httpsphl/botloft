// Getting from "app opened" to "connected to the daemon" without asking
// the owner anything: install it when it does not run, keep it at the
// version this app ships, read the owner token and say hello (spec 15.1).
// Each run installs or updates at most once, so a failure shows instead
// of looping. With a server chosen (spec 42.6), nothing runs here: the app
// goes through the tunnel with this computer's own token there, opening
// that tunnel itself when the server's key says how (`toServer`).

import { createStore, type StoreApi } from "zustand/vanilla";
import { t } from "../../i18n";
import { errorText } from "../../lib/api";
import type { Client } from "../../lib/client";
import type { DaemonStatus, Host } from "../../lib/host";
import { PROTOCOL_VERSION } from "../../lib/protocol.gen";
import { prefs } from "../../shell/prefs";
import { server } from "../../shell/server";
import type { Connect, LinkStep } from "./steps";
import { type Run, toServer } from "./toServer";

export type { Connect, LinkStep } from "./steps";

export interface Link {
  current: LinkStep;
  /** Checks the daemon again from the start, installing it if needed. */
  check(): Promise<void>;
  /** Installs the daemon as a scheduled task and starts it, then connects. */
  install(): Promise<void>;
  /** Restarts the daemon, then connects. */
  restart(): Promise<void>;
}

export function createLink(host: Host, connect: Connect): StoreApi<Link> {
  let api: Client | null = null;
  let generation = 0;
  const way = toServer(host, connect);

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
            const action = prefs.whenClosed.get() === "stop" ? "start" : "install";
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
      way.begin(server.get());
      drop();
      return generation;
    };

    /** What run `run` lends the way to a server. */
    const lend = (run: number): Run => ({
      live: () => run === generation,
      show: (current) => set({ current }),
      keep: (client) => {
        api = client;
      },
      drop,
      again: () => void check(),
    });

    const check = async () => {
      const run = begin();
      const remote = server.get();
      if (remote) {
        await way.reach(remote, lend(run));
        return;
      }
      set({ current: { step: "checking" } });
      try {
        await settle(await host.daemonStatus(), run);
      } catch (error) {
        if (run === generation) {
          set({ current: { step: "error", message: errorText(error) } });
        }
      }
    };

    return {
      current: { step: "checking" },
      check,
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
