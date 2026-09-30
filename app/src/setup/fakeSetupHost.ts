// A `SetupHost` in memory, for tests and the dev preview (`/setup.html`,
// with `?relation=older&installed=0.4.0&running&fail` to try each page).

import type { InstallFailure, Relation, SetupHost, SetupState } from "./host";

export interface FakeSetupOptions {
  relation?: Relation;
  installed?: string | null;
  appRunning?: boolean;
  /** Makes `install` fail with this. */
  failure?: InstallFailure | null;
  /** How long `install` takes, in ms. */
  delay?: number;
}

export class FakeSetupHost implements SetupHost {
  readonly calls: string[] = [];
  private readonly current: SetupState;
  private failure: InstallFailure | null;
  private readonly delay: number;

  constructor(options: FakeSetupOptions = {}) {
    const relation = options.relation ?? "none";
    this.current = {
      version: "0.6.0",
      rehearsal: true,
      folder: "C:UsersAna LimaAppDataLocalBotloft",
      installed: relation === "none" ? null : (options.installed ?? "0.5.0"),
      relation,
      appRunning: options.appRunning ?? false,
    };
    this.failure = options.failure ?? null;
    this.delay = options.delay ?? 0;
  }

  /** Lets the next `install` succeed. */
  succeed(): void {
    this.failure = null;
  }

  async state(): Promise<SetupState> {
    this.calls.push("state");
    return { ...this.current };
  }

  async install(): Promise<void> {
    this.calls.push("install");
    await new Promise((resolve) => setTimeout(resolve, this.delay));
    if (this.failure) {
      throw this.failure;
    }
  }

  async openApp(): Promise<void> {
    this.calls.push("openApp");
  }

  async classic(): Promise<void> {
    this.calls.push("classic");
  }

  close(): void {
    this.calls.push("close");
  }

  show(): void {
    this.calls.push("show");
  }
}

/** The dev preview's fake, set up from the page's query string. */
export function previewSetupHost(search: string): SetupHost {
  const params = new URLSearchParams(search);
  const relation = params.get("relation");
  return new FakeSetupHost({
    relation:
      relation === "older" || relation === "same" || relation === "newer" ? relation : "none",
    installed: params.get("installed"),
    appRunning: params.has("running"),
    failure: params.has("fail")
      ? { code: 2, detail: "Botloft-installer.exe /S stopped with exit code: 2" }
      : null,
    delay: 3000,
  });
}
