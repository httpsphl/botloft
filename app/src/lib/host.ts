// What the app needs from the native shell (spec 15.2): the daemon's
// status, installing and restarting it, the owner token, Explorer and the
// window. The Tauri implementation lives in `tauriHost.ts`; tests pass a
// fake.

export type DaemonStatus =
  | {
      state: "running";
      port: number;
      version: string;
      protocol: number;
      /** Older than the daemon this app ships; installing replaces it. */
      outdated: boolean;
    }
  /** Nothing listens on the port. `home` is the daemon's data folder. */
  | { state: "stopped"; port: number; home: string }
  /** Something that is not botloftd holds the port. */
  | { state: "foreign"; port: number };

/** A newer Botloft, ready to download and install (spec 15.5). */
export interface AppUpdate {
  version: string;
  /** Release notes, if the release has any. */
  notes: string | null;
  /**
   * Downloads and installs the update. Botloft closes, the installer runs
   * and opens it again, so this only returns if something fails.
   * `progress` gets the downloaded fraction, or null while the size is
   * unknown.
   */
  install(progress: (fraction: number | null) => void): Promise<void>;
}

export interface AppWindow {
  minimize(): Promise<void>;
  toggleMaximize(): Promise<void>;
  close(): Promise<void>;
  isMaximized(): Promise<boolean>;
  /** Called when the window is resized; returns the unsubscribe function. */
  onResized(listener: () => void): Promise<() => void>;
  /** Marks the taskbar icon while something waits for the owner. */
  setAttention(on: boolean): Promise<void>;
}

export interface Host {
  daemonStatus(): Promise<DaemonStatus>;
  /**
   * Installs the daemon this app ships as a scheduled task that starts at
   * logon, starts it and waits for it to answer (spec 14). Replaces an
   * older daemon; leaves the same version running.
   */
  installDaemon(): Promise<DaemonStatus>;
  /** Stops the daemon and starts it again from its scheduled task. */
  restartDaemon(): Promise<DaemonStatus>;
  /**
   * Opens Claude Code's sign-in (`claude auth login`) in its own window and
   * waits for it to end. Resolves whether it signed in.
   */
  signInToClaude(claudePath: string): Promise<boolean>;
  /** Draws the whole window at `factor` times its size (1 = 100%). */
  setZoom(factor: number): Promise<void>;
  /** A newer version of the app, or null when this one is the latest. */
  checkForUpdate(): Promise<AppUpdate | null>;
  readOwnerToken(): Promise<string>;
  /** Opens a folder in Explorer. */
  openPath(path: string): Promise<void>;
  /**
   * Asks the owner for a folder with Windows' folder picker, starting at
   * `start` if given. Resolves null if they cancel.
   */
  pickFolder(title: string, start?: string): Promise<string | null>;
  /** Opens an http(s) link in the default browser. */
  openUrl(url: string): Promise<void>;
  window: AppWindow;
}
