// The fake daemon's browsers (spec 21.7): each bot's state, the live frames
// of the one being watched and what the bot does, for the browser panel,
// and the owner's hands in it (spec 21.10).

import type { FakeBotloft, Handlers } from "./fake";
import { conflict } from "./fakeRules";
import type {
  ApprovalId,
  BotId,
  BrowserAction,
  BrowserActionKind,
  BrowserFrame,
  BrowserInput,
  BrowserState,
} from "./protocol.gen";

export class FakeBrowser {
  private readonly states = new Map<BotId, BrowserState>();
  private readonly frames = new Map<BotId, BrowserFrame>();
  /** The bot whose browser the app watches, if any. */
  watching: BotId | null = null;
  /** Every watch and unwatch, in order. */
  readonly watches: (BotId | null)[] = [];
  /** What the owner sent to the page, in order. */
  readonly inputs: BrowserInput[] = [];
  /** Each bot's open request for help. */
  private readonly asks = new Map<BotId, ApprovalId>();
  /** Pages that answer the owner's hands, for the preview. */
  private readonly pages = new Map<BotId, (input: BrowserInput) => void>();

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
        control: "bot",
        ask: null,
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
      control: "bot",
      ask: null,
    });
  }

  /** The bot asks the owner for a hand: a request in the chat and in the panel. */
  ask(botId: BotId, task: string): ApprovalId {
    const url = this.state(botId).url;
    const input = JSON.stringify({ task, url, site: url ? new URL(url).hostname : null });
    const item = this.fake.chat.ask(botId, "mcp__botloft__browser_help", task, input);
    const approvalId = item.body.kind === "approval" ? item.body.approvalId : "";
    this.asks.set(botId, approvalId);
    this.set(botId, { ask: task });
    return approvalId;
  }

  /** Calls `page` with each event the owner sends to the bot's browser. */
  onInput(botId: BotId, page: (input: BrowserInput) => void): void {
    this.pages.set(botId, page);
  }

  /** The request for help was answered: the browser goes back to the bot. */
  helped(botId: BotId): void {
    this.asks.delete(botId);
    this.set(botId, { ask: null, control: "bot" });
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

  /** The watched browser leaves the owner's hands, if it was in them. */
  private release(): BrowserState | null {
    const botId = this.watching;
    if (botId === null || this.state(botId).control !== "owner") {
      return null;
    }
    return this.set(botId, { control: "bot" });
  }

  handlers(): Pick<
    Handlers,
    | "browser.list"
    | "browser.watch"
    | "browser.unwatch"
    | "browser.take"
    | "browser.release"
    | "browser.input"
  > {
    return {
      "browser.list": () => [...this.states.values()].filter((state) => state.status !== "closed"),
      "browser.watch": ({ botId }) => {
        this.fake.bot(botId, false);
        this.release();
        this.watching = botId;
        this.watches.push(botId);
        return { state: this.state(botId), frame: this.frames.get(botId) ?? null };
      },
      "browser.unwatch": () => {
        this.release();
        this.watching = null;
        this.watches.push(null);
        return null;
      },
      "browser.take": ({ botId }) => {
        if (this.watching !== botId) {
          throw conflict("watch this browser before taking it");
        }
        if (this.state(botId).status !== "open") {
          throw conflict("the browser is not open");
        }
        return this.set(botId, { control: "owner" });
      },
      "browser.release": ({ botId }) => {
        const approvalId = this.asks.get(botId);
        if (this.state(botId).control === "owner" && approvalId) {
          void this.fake.call("approvals.answer", { approvalId, allow: true });
        }
        return this.release() ?? this.state(botId);
      },
      "browser.input": ({ botId, input }) => {
        if (this.watching !== botId || this.state(botId).control !== "owner") {
          throw conflict("take the browser before using it");
        }
        this.inputs.push(input);
        this.pages.get(botId)?.(input);
        return null;
      },
    };
  }
}
