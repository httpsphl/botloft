// A `Host` for tests: a daemon that is running unless told otherwise.

import type { DaemonStatus, Host } from "./host";
import { PROTOCOL_VERSION } from "./protocol.gen";

export class FakeHost implements Host {
  status: DaemonStatus = {
    state: "running",
    port: 45710,
    version: "0.1.0",
    protocol: PROTOCOL_VERSION,
  };
  /** What `startDaemon` leaves behind. */
  afterStart: DaemonStatus = {
    state: "running",
    port: 45710,
    version: "0.1.0",
    protocol: PROTOCOL_VERSION,
  };
  token: string | Error = "a".repeat(64);
  readonly opened: string[] = [];
  starts = 0;
  maximized = false;
  closed = false;
  attention = false;

  daemonStatus(): Promise<DaemonStatus> {
    return Promise.resolve(this.status);
  }

  startDaemon(): Promise<DaemonStatus> {
    this.starts += 1;
    this.status = this.afterStart;
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
