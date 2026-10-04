// The fake daemon's desktop (spec 24): what each bot may see and do on the
// owner's desktop (spec 24.2, 24.10), what it is doing there for its panel,
// with pictures while the panel watches, and stopping it (spec 24.9). The
// bots' desktop tools are not played: grants come from `grant` and use
// from `use`, as the real daemon makes them.

import type { FakeBotloft, Handlers } from "./fake";
import { notFound } from "./fakeRules";
import type {
  BotId,
  ChatItemId,
  DesktopAction,
  DesktopAwayUse,
  DesktopFrame,
  DesktopGrant,
  DesktopLevel,
  DesktopState,
  DesktopWindow,
} from "./protocol.gen";

type DesktopMethods = Extract<keyof Handlers, `desktop.${string}`>;

export class FakeDesktop {
  readonly grants: DesktopGrant[] = [];
  private readonly states = new Map<BotId, DesktopState>();
  private readonly frames = new Map<BotId, DesktopFrame>();
  /** What bots did while the owner was away. */
  away: DesktopAwayUse[] = [];
  /** The bot whose panel the app watches, if any. */
  watching: BotId | null = null;

  constructor(private readonly fake: FakeBotloft) {}

  /** Lets the bot see, or see and act, in an app, and tells the app. */
  grant(botId: BotId, appPath: string, appName: string, level: DesktopLevel = "see"): DesktopGrant {
    const same = this.grants.find(
      (grant) => grant.botId === botId && grant.appPath?.toLowerCase() === appPath.toLowerCase(),
    );
    const grant: DesktopGrant = same ?? {
      id: this.fake.id("dsk"),
      botId,
      scope: "app",
      appPath,
      appName,
      level,
      realInput: false,
      unattended: false,
      acceptedRisksAt: null,
      createdAt: this.fake.now,
    };
    if (same) {
      same.level = same.level === "act" || level === "act" ? "act" : "see";
    } else {
      this.grants.push(grant);
    }
    this.changed(botId);
    return grant;
  }

  private of(botId: BotId): DesktopGrant[] {
    return this.grants.filter((grant) => grant.botId === botId);
  }

  private changed(botId: BotId) {
    this.fake.emit({ name: "bot.desktop", params: { botId, grants: this.of(botId) } });
  }

  state(botId: BotId): DesktopState {
    return (
      this.states.get(botId) ?? { botId, window: null, action: null, at: null, stopped: false }
    );
  }

  private set(botId: BotId, change: Partial<DesktopState>): DesktopState {
    const state = { ...this.state(botId), ...change };
    this.states.set(botId, state);
    this.fake.emit({ name: "desktop.changed", params: state });
    return state;
  }

  /** The bot read or acted in `window`; `action` says what it did. */
  use(botId: BotId, window: DesktopWindow, action: DesktopAction | null = null): DesktopState {
    return this.set(botId, { window, action, at: this.fake.now });
  }

  /** The bot used `app` while the owner was away (spec 24.8). */
  usedAway(botId: BotId, app: string, itemId: ChatItemId | null = null): void {
    const same = this.away.find((used) => used.botId === botId && used.app === app);
    if (same) {
      same.until = this.fake.now;
    } else {
      this.away.push({ botId, app, from: this.fake.now, until: this.fake.now, itemId });
    }
    this.fake.emit({ name: "desktop.away", params: [...this.away] });
  }

  /** A new picture of the bot's window: sent while its panel watches. */
  paint(botId: BotId, data: string, width = 800, height = 600): void {
    const frame = { botId, data, width, height };
    this.frames.set(botId, frame);
    if (this.watching === botId) {
      this.fake.emit({ name: "desktop.frame", params: frame });
    }
  }

  handlers(): Pick<Handlers, DesktopMethods> {
    return {
      "desktop.grants": ({ botId }) => {
        this.fake.bot(botId);
        return this.of(botId);
      },
      "desktop.revoke": ({ grantId }) => {
        const at = this.grants.findIndex((grant) => grant.id === grantId);
        const grant = this.grants[at];
        if (!grant) {
          throw notFound(`desktop grant ${grantId}`);
        }
        this.grants.splice(at, 1);
        this.changed(grant.botId);
        return { botId: grant.botId, grants: this.of(grant.botId) };
      },
      "desktop.setOptions": ({ grantId, realInput, unattended, acceptedRisks }) => {
        const grant = this.grants.find((each) => each.id === grantId);
        if (!grant) {
          throw notFound(`desktop grant ${grantId}`);
        }
        if (unattended && !acceptedRisks) {
          throw new Error("use while the owner is away needs the risks accepted first");
        }
        if (realInput !== undefined) {
          grant.realInput = realInput;
        }
        if (unattended !== undefined) {
          grant.unattended = unattended;
          grant.acceptedRisksAt = unattended ? this.fake.now : grant.acceptedRisksAt;
        }
        this.changed(grant.botId);
        return { botId: grant.botId, grants: this.of(grant.botId) };
      },
      "desktop.watch": ({ botId }) => {
        this.fake.bot(botId);
        this.watching = botId;
        return { state: this.state(botId), frame: this.frames.get(botId) ?? null };
      },
      "desktop.unwatch": () => {
        this.watching = null;
        return null;
      },
      "desktop.stop": ({ botId }) => {
        this.fake.bot(botId);
        return this.set(botId, { stopped: true });
      },
      "desktop.resume": ({ botId }) => {
        this.fake.bot(botId);
        return this.set(botId, { stopped: false });
      },
      "desktop.awayUses": () => this.away,
      "desktop.dismissAway": () => {
        this.away = [];
        this.fake.emit({ name: "desktop.away", params: [] });
        return null;
      },
    };
  }
}
