// A `Host` for tests: a daemon that is running unless told otherwise.

import type { DaemonStatus, Host } from "./host";
import { PROTOCOL_VERSION } from "./protocol.gen";

/** A current daemon on the default port. */
export function running(
  changes: Partial<Extract<DaemonStatus, { state: "running" }>> = {},
): DaemonStatus {
  return {
    state: "running",
    port: 45710,
    version: "0.1.0",
    protocol: PROTOCOL_VERSION,
    outdated: false,
    ...changes,
  };
}

export class FakeHost implements Host {
  status: DaemonStatus = running();
  /** What `installDaemon` and `restartDaemon` leave behind, or their error. */
  afterInstall: DaemonStatus | Error = running();
  token: string | Error = "a".repeat(64);
  readonly opened: string[] = [];
  readonly installs: ("install" | "restart")[] = [];
  maximized = false;
  closed = false;
  attention = false;

  daemonStatus(): Promise<DaemonStatus> {
    return Promise.resolve(this.status);
  }

  installDaemon(): Promise<DaemonStatus> {
    return this.settle("install");
  }

  restartDaemon(): Promise<DaemonStatus> {
    return this.settle("restart");
  }

  private settle(action: "install" | "restart"): Promise<DaemonStatus> {
    this.installs.push(action);
    if (this.afterInstall instanceof Error) {
      return Promise.reject(this.afterInstall);
    }
    this.status = this.afterInstall;
    return Promise.resolve(this.status);
  }

  readOwnerToken(): Promise<string> {
    return this.token instanceof Error ? Promise.reject(this.token) : Promise.resolve(this.token);
  }

  openPath(path: string): Promise<void> {
    this.opened.push(path);
    return Promise.resolve();
  }

  openUrl(url: string): Promise<void> {
    this.opened.push(url);
    return Promise.resolve();
  }

  window = {
    minimize: () => Promise.resolve(),
    toggleMaximize: () => {
      this.maximized = !this.maximized;
      return Promise.resolve();
    },
    close: () => {
      this.closed = true;
      return Promise.resolve();
    },
    isMaximized: () => Promise.resolve(this.maximized),
    onResized: () => Promise.resolve(() => {}),
    setAttention: (on: boolean) => {
      this.attention = on;
      return Promise.resolve();
    },
  };
}
