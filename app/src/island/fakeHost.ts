// The island's native shell, for tests and the preview: it remembers what
// the island asked, and the test sets whether the screen is quiet or the
// main window is in front.

import type { IslandHost, IslandScreen } from "./host";

export class FakeIslandHost implements IslandHost {
  /** What the island asked, in order: "show 200x36 primary", "hide", "open bot_1". */
  readonly calls: string[] = [];
  readonly supported = true;
  /** Windows would hold back a notification. */
  quietNow = false;
  /** The main window is on screen with the focus. */
  front = false;

  daemon(): Promise<{ port: number; token: string }> {
    return Promise.resolve({ port: 0, token: "" });
  }

  show(width: number, height: number, screen: IslandScreen): Promise<void> {
    this.calls.push(`show ${width}x${height} ${screen}`);
    return Promise.resolve();
  }

  hide(): Promise<void> {
    this.calls.push("hide");
    return Promise.resolve();
  }

  openApp(botId?: string): Promise<void> {
    this.calls.push(`open ${botId ?? ""}`);
    return Promise.resolve();
  }

  quiet(): Promise<boolean> {
    return Promise.resolve(this.quietNow);
  }

  appInFront(): Promise<boolean> {
    return Promise.resolve(this.front);
  }

  private readonly summons = new Set<() => void>();
  private readonly fronts = new Set<(front: boolean) => void>();

  onSummon(listener: () => void): Promise<() => void> {
    this.summons.add(listener);
    return Promise.resolve(() => this.summons.delete(listener));
  }

  onFront(listener: (front: boolean) => void): Promise<() => void> {
    this.fronts.add(listener);
    return Promise.resolve(() => this.fronts.delete(listener));
  }

  setSummon(on: boolean, holdMs: number): Promise<void> {
    this.calls.push(`summon ${on ? holdMs : "off"}`);
    return Promise.resolve();
  }

  private readonly locales = new Set<(locale: string) => void>();

  onLocale(listener: (locale: string) => void): Promise<() => void> {
    this.locales.add(listener);
    return Promise.resolve(() => this.locales.delete(listener));
  }

  /** The main window says which language it shows. */
  tellLocale(locale: string): void {
    for (const listener of this.locales) {
      listener(locale);
    }
  }

  askLocale(): Promise<void> {
    return Promise.resolve();
  }

  /** The owner held the right Ctrl long enough. */
  summon(): void {
    for (const listener of this.summons) {
      listener();
    }
  }

  /** The main window came to the front, or left it. */
  setFront(front: boolean): void {
    this.front = front;
    for (const listener of this.fronts) {
      listener(front);
    }
  }
}
