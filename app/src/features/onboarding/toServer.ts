// The link's way to a server (spec 42.6): through a tunnel that ends on this
// computer, the app's own when the server's key says how to open it (the
// app keeps it open), else one the owner opened. Nothing runs here.

import type { Client } from "../../lib/client";
import type { Host, TunnelSpec } from "../../lib/host";
import { type ServerProfile, server } from "../../shell/server";
import { mayPass, setTunnelPort, tunnelProblem, tunnelSpec } from "../../shell/tunnel";
import type { Connect, LinkStep } from "./steps";

/** How long the app waits to open its tunnel again after the server did not answer. */
export const TUNNEL_RETRY_MS = 10_000;

/** What one run of the link lends the way to a server. */
export interface Run {
  /** Whether this run is still the link's own. */
  live(): boolean;
  show(step: LinkStep): void;
  /** The client the link closes when it starts over. */
  keep(client: Client): void;
  drop(): void;
  /** Starts the link over. */
  again(): void;
}

export function toServer(host: Host, connect: Connect) {
  let retry: ReturnType<typeof setTimeout> | undefined;
  /** Whether the app's own tunnel may run, to close it when the server goes. */
  let tunnelOpen = false;

  /** Closes the app's own tunnel when `remote` (the server chosen now) has none. */
  const leave = (remote: ServerProfile | null) => {
    if (tunnelOpen && !remote?.ssh) {
      tunnelOpen = false;
      setTunnelPort(null);
      void host.tunnel.close().catch(() => undefined);
    }
  };

  /** Opens the app's own tunnel to `remote`: its port, or null when it did not open. */
  const open = async (remote: ServerProfile, spec: TunnelSpec, run: Run) => {
    try {
      const port = await host.tunnel.open(spec);
      tunnelOpen = true;
      if (!run.live()) {
        // The owner moved on while it opened, maybe back to this computer.
        leave(server.get());
        return null;
      }
      setTunnelPort(port);
      return port;
    } catch (error) {
      if (run.live()) {
        const problem = tunnelProblem(error);
        run.show({ step: "unreachable", server: remote, problem });
        if (mayPass(problem)) {
          retry = setTimeout(() => {
            if (run.live()) {
              void reach(remote, run);
            }
          }, TUNNEL_RETRY_MS);
        }
      }
      return null;
    }
  };

  /** Through the tunnel to `remote`, which keeps trying until it answers. */
  const reach = async (remote: ServerProfile, run: Run) => {
    const spec = tunnelSpec(remote);
    let port = remote.tunnel;
    if (spec) {
      run.show({ step: "opening", server: remote });
      const ready = await open(remote, spec, run);
      if (ready === null || !run.live()) {
        return;
      }
      port = ready;
    }
    const client = connect(port, remote.token);
    run.keep(client);
    let opened = false;
    let reopening = false;
    // After the network went away, `ssh` may have stopped: the tunnel opens
    // again (the same one while it runs), and on another port the link
    // starts over.
    const reopen = (own: TunnelSpec) => {
      reopening = true;
      host.tunnel.open(own).then(
        (again) => {
          reopening = false;
          if (run.live() && again !== port) {
            run.again();
          }
        },
        () => {
          reopening = false;
        },
      );
    };
    const follow = () => {
      const connection = client.connection();
      if (!run.live()) {
        return;
      }
      if (connection.kind === "open") {
        opened = true;
        run.show({ step: "connected", api: client });
      } else if (connection.kind === "failed") {
        run.drop();
        run.show({ step: "refused", reason: connection.reason, server: remote });
      } else if (connection.kind === "waiting") {
        if (!spec && !opened) {
          run.show({ step: "unreachable", server: remote });
        }
        if (spec && !reopening) {
          reopen(spec);
        }
      }
    };
    run.show({ step: "connecting", api: client, server: remote });
    client.onConnection(follow);
    follow();
  };

  return {
    reach,
    /**
     * A new run of the link: the last one's retry stops, and the app's own
     * tunnel closes when no server needs it.
     */
    begin(remote: ServerProfile | null) {
      clearTimeout(retry);
      leave(remote);
    },
  };
}
