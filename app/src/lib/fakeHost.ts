// A `Host` for tests: a daemon that is running unless told otherwise.

import type { AppUpdate, DaemonStatus, Host, TrayActions, TrayView } from "./host";
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
  /** Files opened with their program, and files shown in their folder. */
  readonly openedFiles: string[] = [];
  readonly revealed: string[] = [];
  /** Files saved as a copy, and whether the Save dialog picks a place. */
  readonly savedFiles: string[] = [];
  saveChosen = true;
  /** What the folder picker gives back; null is a cancel. */
  nextFolder: string | null = null;
  /** Where each folder picker started. */
  readonly pickerStarts: (string | null)[] = [];
  readonly installs: ("install" | "restart")[] = [];
  /** How many times the app stopped the daemon. */
  stops = 0;
  /** What runs before the window closes, as the app set it. */
  beforeClose: (() => Promise<"close" | "stay">) | null = null;
  hidden = false;
  /** Whether the window is on screen with the focus. */
  front = true;
  /** Whether Windows opens the app at sign-in, as the app last set it. */
  openAtSignIn: boolean | null = null;
  atSignIn = false;
  readonly notices: { title: string; body: string; sound: boolean }[] = [];
  /** The icon near the clock, while shown, and what its menu does. */
  tray: { view: TrayView; actions: TrayActions } | null = null;
  private readonly reopenListeners = new Set<() => void>();
  /** Paths `signInToClaude` ran, and what it answers. */
  readonly signIns: string[] = [];
  signInResult: boolean | Error = true;
  /** Runs when a sign-in ends, before it resolves (e.g. to sign the fake daemon in). */
  onSignIn: () => void = () => {};
  /** What `checkForUpdate` finds, or its error. */
  update: AppUpdate | null | Error = null;
  updateChecks = 0;
  /** The window's size, as the app last set it. */
  zoom = 1;
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

  stopDaemon(): Promise<void> {
    this.stops += 1;
    return Promise.resolve();
  }

  /** Closes the window the way Windows does: what the app set runs first. */
  async requestClose(): Promise<void> {
    const outcome = await this.beforeClose?.();
    if (outcome === "stay") {
      this.hidden = true;
      this.front = false;
    } else {
      this.closed = true;
    }
  }

  /** Botloft opened again from the Start menu or a notification. */
  reopen(): void {
    this.hidden = false;
    this.front = true;
    for (const listener of this.reopenListeners) {
      listener();
    }
  }

  setOpenAtSignIn(on: boolean): Promise<void> {
    this.openAtSignIn = on;
    return Promise.resolve();
  }

  launchedAtSignIn(): Promise<boolean> {
    return Promise.resolve(this.atSignIn);
  }

  notify(notice: { title: string; body: string; sound: boolean }): Promise<void> {
    this.notices.push(notice);
    return Promise.resolve();
  }

  showTray(view: TrayView, actions: TrayActions): Promise<void> {
    this.tray = { view, actions };
    return Promise.resolve();
  }

  hideTray(): Promise<void> {
    this.tray = null;
    return Promise.resolve();
  }

  signInToClaude(claudePath: string): Promise<boolean> {
    this.signIns.push(claudePath);
    if (this.signInResult instanceof Error) {
      return Promise.reject(this.signInResult);
    }
    this.onSignIn();
    return Promise.resolve(this.signInResult);
  }

  setZoom(factor: number): Promise<void> {
    this.zoom = factor;
    return Promise.resolve();
  }

  checkForUpdate(): Promise<AppUpdate | null> {
    this.updateChecks += 1;
    return this.update instanceof Error
      ? Promise.reject(this.update)
      : Promise.resolve(this.update);
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

  openFile(path: string): Promise<void> {
    this.openedFiles.push(path);
    return Promise.resolve();
  }

  revealFile(path: string): Promise<void> {
    this.revealed.push(path);
    return Promise.resolve();
  }

  saveFileAs(path: string): Promise<boolean> {
    this.savedFiles.push(path);
    return Promise.resolve(this.saveChosen);
  }

  pickFolder(_title: string, start?: string): Promise<string | null> {
    this.pickerStarts.push(start ?? null);
    return Promise.resolve(this.nextFolder);
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
    close: () => this.requestClose(),
    hide: () => {
      this.hidden = true;
      this.front = false;
      return Promise.resolve();
    },
    show: () => {
      this.hidden = false;
      this.front = true;
      return Promise.resolve();
    },
    quit: () => {
      this.closed = true;
      return Promise.resolve();
    },
    inFront: () => this.front,
    onReopened: (listener: () => void) => {
      this.reopenListeners.add(listener);
      return Promise.resolve(() => {
        this.reopenListeners.delete(listener);
      });
    },
    onCloseRequested: (before: () => Promise<"close" | "stay">) => {
      this.beforeClose = before;
      return Promise.resolve(() => {
        if (this.beforeClose === before) {
          this.beforeClose = null;
        }
      });
    },
    isMaximized: () => Promise.resolve(this.maximized),
    onResized: () => Promise.resolve(() => {}),
    setAttention: (on: boolean) => {
      this.attention = on;
      return Promise.resolve();
    },
  };
}

/** An update whose install reports some progress, then fails or hangs. */
export class FakeUpdate implements AppUpdate {
  notes: string | null = null;
  installs = 0;
  /** Rejects the install with this, or never settles, as a real one exits. */
  failure: Error | null = null;

  constructor(readonly version: string) {}

  install(progress: (fraction: number | null) => void): Promise<void> {
    this.installs += 1;
    progress(null);
    progress(0.5);
    return this.failure ? Promise.reject(this.failure) : new Promise(() => {});
  }
}
