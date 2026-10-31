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
}
