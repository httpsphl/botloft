// The fake daemon's design area (spec 22): a bot's HTML screens and the
// drafts it streams while writing one. Addresses are `data:` URLs, so the
// preview renders them without a daemon.

import type { FakeBotloft, Handlers } from "./fake";
import type { BotId, Screen, ScreenDevice } from "./protocol.gen";

const pageUrl = (html: string) => `data:text/html;charset=utf-8,${encodeURIComponent(html)}`;

export class FakeScreens {
  private readonly screens = new Map<BotId, Screen[]>();
  private readonly revs = new Map<string, number>();

  constructor(private readonly fake: FakeBotloft) {}

  /** A screen on disk. `folder` is relative to the work folder. */
  add(
    botId: BotId,
    name: string,
    html: string,
    options: { folder?: string; device?: ScreenDevice; at?: number } = {},
  ): Screen {
    this.fake.bot(botId);
    const folder = options.folder ?? "";
    const screen: Screen = {
      path: `C:\\Work\\${folder ? `${folder}\\` : ""}${name}`,
      name,
      folder,
      modifiedAt: options.at ?? this.fake.now,
      url: pageUrl(html),
      device: options.device ?? null,
      writing: false,
    };
    const list = (this.screens.get(botId) ?? []).filter((other) => other.path !== screen.path);
    this.screens.set(botId, [screen, ...list]);
    return screen;
  }

  /** A new version of a screen the bot is writing; `done` ends it. */
  draft(botId: BotId, path: string, html: string, done = false): void {
    const rev = (this.revs.get(path) ?? 0) + 1;
    this.revs.set(path, rev);
    this.fake.emit({
      name: "screen.draft",
      params: { botId, path, url: pageUrl(html), rev, bytes: html.length, done },
    });
  }

  handlers(): Pick<Handlers, "screens.list"> {
    return {
      "screens.list": ({ botId }) => {
        this.fake.bot(botId, false);
        return [...(this.screens.get(botId) ?? [])].sort((a, b) => b.modifiedAt - a.modifiedAt);
      },
    };
  }
}
