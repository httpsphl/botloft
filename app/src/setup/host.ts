// What the setup window needs from its host (spec 15.7): the commands in
// `src-setup/src/main.rs`, or a fake for tests and the dev preview.

/** The installed Botloft next to the one this setup carries. */
export type Relation = "none" | "older" | "same" | "newer";

export interface SetupState {
  /** The version this setup installs. */
  version: string;
  /** No installer inside (a dev build): the pages only pretend. */
  rehearsal: boolean;
  /** Where Botloft goes. */
  folder: string | null;
  /** The version already installed. */
  installed: string | null;
  relation: Relation;
  /** The app is open and closes to install. */
  appRunning: boolean;
}

/** Why the installer did not finish. */
export interface InstallFailure {
  /** The installer's exit code, when it ran. */
  code: number | null;
  detail: string;
}

export interface SetupHost {
  state(): Promise<SetupState>;
  /** Runs the installer quietly; rejects with an `InstallFailure`. */
  install(): Promise<void>;
  /** Opens the installed app and closes the setup. */
  openApp(): Promise<void>;
  /** Hands over to the installer's own pages and closes the setup. */
  classic(): Promise<void>;
  close(): void;
  /** Shows the window once the first page is drawn. */
  show(): void;
}

/** A rejection from `install`, whatever shape it came in. */
export function asFailure(error: unknown): InstallFailure {
  if (typeof error === "object" && error !== null && "detail" in error) {
    const { code, detail } = error as { code?: unknown; detail?: unknown };
    return { code: typeof code === "number" ? code : null, detail: String(detail) };
  }
  return { code: null, detail: String(error) };
}
