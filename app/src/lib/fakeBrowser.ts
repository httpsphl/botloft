// The fake daemon's browsers (spec 21.7): each bot's state and tabs, the
// live frames of the one being watched and what the bot does, for the
// browser panel, the size the panel gives the page (spec 21.3), and the
// owner's hands in it (spec 21.10).

import type { FakeBotloft, Handlers } from "./fake";
import { fitting, PAGE, type PageSize, typed } from "./fakePages";
import { conflict, invalid, notFound } from "./fakeRules";
import type {
  ApprovalId,
  BotId,
  BrowserAction,
  BrowserActionKind,
  BrowserFrame,
  BrowserInput,
  BrowserState,
  BrowserTab,
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
  /** Each time the owner reloaded a bot's page. */
  readonly reloads: BotId[] = [];
  /** Each bot's page size, while the panel asks for one. */
  private readonly sizes = new Map<BotId, PageSize>();
  /** What draws each bot's page, for the preview. */
  private readonly painters = new Map<BotId, (size: PageSize) => string>();
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
        tabs: [],
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

  /** Puts `tabs` in the bot's browser, the active one's page on top. */
  private setTabs(botId: BotId, tabs: BrowserTab[], more: Partial<BrowserState> = {}) {
    const active = tabs.find((tab) => tab.active);
    const page = { url: active?.url ?? null, title: active?.title || null };
    return this.set(botId, { tabs, ...page, ...more });
  }

  /** A page opened in the active tab; the first tab, if there is none. */
  open(botId: BotId, url: string, title: string): BrowserState {
    const tabs = this.state(botId).tabs;
    const next =
      tabs.length === 0
        ? [{ id: this.fake.id("tab"), url, title, active: true }]
        : tabs.map((tab) => (tab.active ? { ...tab, url, title } : tab));
    return this.setTabs(botId, next, { status: "open", loading: false, error: null });
  }

  /** Another tab opened, and became the active one. */
  openTab(botId: BotId, url: string, title: string): BrowserTab {
    const tab = { id: this.fake.id("tab"), url, title, active: true };
    const others = this.state(botId).tabs.map((other) => ({ ...other, active: false }));
    this.setTabs(botId, [...others, tab]);
    return tab;
  }

  close(botId: BotId): BrowserState {
    this.frames.delete(botId);
    return this.setTabs(botId, [], { status: "closed", loading: false, control: "bot", ask: null });
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

  /** The size of the bot's page now. */
  size(botId: BotId): PageSize {
    return this.sizes.get(botId) ?? PAGE;
  }

  /** Draws the bot's page with `draw`, now and whenever its size changes. */
  paint(botId: BotId, draw: (size: PageSize) => string): void {
    this.painters.set(botId, draw);
    this.repaint(botId);
  }

  /** The page changed: a new picture of it, if something draws it. */
  repaint(botId: BotId): void {
    const draw = this.painters.get(botId);
    if (draw) {
      this.frame(botId, draw(this.size(botId)));
    }
  }

  /** A new picture of the page; sent only while the app watches that bot. */
  frame(botId: BotId, data: string, size: PageSize = this.size(botId)): void {
    const frame: BrowserFrame = { botId, data, ...size };
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

  /** The owner has the watched browser, or the call is refused. */
  private held(botId: BotId): void {
    if (this.watching !== botId || this.state(botId).control !== "owner") {
      throw conflict("take the browser before using it");
    }
  }

  /** Nobody watches: the page goes back to the size bots work in alone. */
  private unwatch(): void {
    const botId = this.watching;
    this.release();
    this.watching = null;
    if (botId !== null && this.sizes.delete(botId)) {
      this.repaint(botId);
    }
  }

  handlers(): Pick<
    Handlers,
    | "browser.list"
    | "browser.watch"
    | "browser.unwatch"
    | "browser.resize"
    | "browser.take"
    | "browser.release"
    | "browser.input"
    | "browser.reload"
    | "browser.newTab"
    | "browser.switchTab"
    | "browser.open"
  > {
    return {
      "browser.list": () => [...this.states.values()].filter((state) => state.status !== "closed"),
      "browser.watch": ({ botId }) => {
        this.fake.bot(botId, false);
        this.unwatch();
        this.watching = botId;
        this.watches.push(botId);
        return { state: this.state(botId), frame: this.frames.get(botId) ?? null };
      },
      "browser.unwatch": () => {
        this.unwatch();
        this.watches.push(null);
        return null;
      },
      "browser.resize": ({ botId, width, height }) => {
        if (this.watching !== botId) {
          throw conflict("watch this browser before sizing it");
        }
        this.sizes.set(botId, fitting(width, height));
        this.repaint(botId);
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
        this.held(botId);
        this.inputs.push(input);
        this.pages.get(botId)?.(input);
        return null;
      },
      "browser.reload": ({ botId }) => {
        if (this.watching !== botId) {
          throw conflict("watch this browser before reloading it");
        }
        if (this.state(botId).status !== "open") {
          throw conflict("the browser is not open");
        }
        this.reloads.push(botId);
        this.repaint(botId);
        return null;
      },
      "browser.newTab": ({ botId }) => {
        this.held(botId);
        this.openTab(botId, "about:blank", "");
        this.repaint(botId);
        return null;
      },
      "browser.switchTab": ({ botId, tabId }) => {
        this.held(botId);
        const tabs = this.state(botId).tabs;
        if (!tabs.some((tab) => tab.id === tabId)) {
          throw notFound("that tab");
        }
        this.setTabs(
          botId,
          tabs.map((tab) => ({ ...tab, active: tab.id === tabId })),
        );
        this.repaint(botId);
        return null;
      },
      "browser.open": ({ botId, url }) => {
        this.held(botId);
        const address = typed(url);
        if (!address) {
          throw invalid("that is not a web address");
        }
        this.open(botId, address, new URL(address).hostname);
        this.repaint(botId);
        return null;
      },
    };
  }
}
