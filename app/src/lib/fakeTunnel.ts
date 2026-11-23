// The app's own SSH tunnel for tests and `pnpm dev` (spec 42.6): it opens at
// once on `port`, or when a test lets it, or fails the way a test says.

import type { HostTunnel, TunnelSpec } from "./host";

export class FakeTunnel implements HostTunnel {
  /** Every spec `open` got, in order. */
  readonly opened: TunnelSpec[] = [];
  closed = 0;
  /** The port on this computer `open` resolves. */
  port = 45720;
  /** What the next `open` rejects with, as the Tauri side says it; null opens. */
  failure: string | null = null;
  /** While true, `open` waits for `release`, as a slow server would. */
  hold = false;
  private readonly held: (() => void)[] = [];

  open(spec: TunnelSpec): Promise<number> {
    this.opened.push(spec);
    if (this.failure !== null) {
      return Promise.reject(this.failure);
    }
    if (this.hold) {
      return new Promise((resolve) => this.held.push(() => resolve(this.port)));
    }
    return Promise.resolve(this.port);
  }

  /** Lets the held `open` calls finish. */
  release(): void {
    this.hold = false;
    for (const finish of this.held.splice(0)) {
      finish();
    }
  }

  close(): Promise<void> {
    this.closed += 1;
    return Promise.resolve();
  }
}
