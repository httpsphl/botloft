// The fake daemon's browsers (spec 21.7): each bot's state, the live frames
// of the one being watched and what the bot does, for the browser panel.

import type { FakeBotloft, Handlers } from "./fake";
import type {
  BotId,
  BrowserAction,
  BrowserActionKind,
  BrowserFrame,
  BrowserState,
} from "./protocol.gen";

export class FakeBrowser {
  private readonly states = new Map<BotId, BrowserState>();
  private readonly frames = new Map<BotId, BrowserFrame>();
  /** The bot whose browser the app watches, if any. */
  watching: BotId | null = null;
  /** Every watch and unwatch, in order. */
  readonly watches: (BotId | null)[] = [];

  constructor(private readonly fake: FakeBotloft) {}

  state(botId: BotId): BrowserState {
    return (
      this.states.get(botId) ?? {
        botId,
        status: "closed",
        url: null,
        title: null,
        loading: false,
        tabs: 0,
        error: null,
        updatedAt: this.fake.now,
      }
    );
  }

  /** Changes the bot's browser and tells the app. */
  set(botId: BotId, change: Partial<BrowserState>): BrowserState {
    this.fake.bot(botId);
    const next = { ...this.state(botId), ...change, updatedAt: this.fake.now };
    this.states.set(botId, next);
    this.fake.emit({ name: "browser.changed", params: next });
    return next;
  }

  /** The bot opened a page. */
  open(botId: BotId, url: string, title: string): BrowserState {
    return this.set(botId, { status: "open", url, title, loading: false, tabs: 1, error: null });
  }

  close(botId: BotId): BrowserState {
    this.frames.delete(botId);
    return this.set(botId, {
      status: "closed",
      url: null,
      title: null,
      loading: false,
      tabs: 0,
    });
  }

  /** A new picture of the page; sent only while the app watches that bot. */
  frame(botId: BotId, data: string, width = 1280, height = 800): void {
    const frame: BrowserFrame = { botId, data, width, height };
    this.frames.set(botId, frame);
    if (this.watching === botId) {
      this.fake.emit({ name: "browser.frame", params: frame });
    }
  }

  /** Something the bot did, for the cursor. */
  act(
    botId: BotId,
    kind: BrowserActionKind,
    options: { x?: number; y?: number; label?: string } = {},
  ): BrowserAction {
    const action: BrowserAction = {
      botId,
      kind,
      x: options.x ?? null,
      y: options.y ?? null,
      label: options.label ?? null,
      at: this.fake.now,
    };
    this.fake.emit({ name: "browser.action", params: action });
    return action;
  }

  handlers(): Pick<Handlers, "browser.list" | "browser.watch" | "browser.unwatch"> {
    return {
      "browser.list": () => [...this.states.values()].filter((state) => state.status !== "closed"),
      "browser.watch": ({ botId }) => {
        this.fake.bot(botId, false);
        this.watching = botId;
        this.watches.push(botId);
        return { state: this.state(botId), frame: this.frames.get(botId) ?? null };
      },
      "browser.unwatch": () => {
        this.watching = null;
        this.watches.push(null);
        return null;
      },
    };
  }
}
