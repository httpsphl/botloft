// What the app needs from the native shell (spec 15.2): the daemon's
// status, starting it, the owner token, Explorer and the window. The Tauri
// implementation lives in `tauriHost.ts`; tests pass a fake.

export type DaemonStatus =
  | { state: "running"; port: number; version: string; protocol: number }
  /** Nothing listens on the port. `home` is the daemon's data folder. */
  | { state: "stopped"; port: number; home: string }
  /** Something that is not botloftd holds the port. */
  | { state: "foreign"; port: number };

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
  /** Starts the daemon if needed and waits for it to answer. */
  startDaemon(): Promise<DaemonStatus>;
  readOwnerToken(): Promise<string>;
  /** Opens a folder in Explorer. */
  openPath(path: string): Promise<void>;
  /** Opens an http(s) link in the default browser. */
  openUrl(url: string): Promise<void>;
  window: AppWindow;
}
