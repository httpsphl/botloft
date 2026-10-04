// The fake daemon's desktop grants (spec 24.2, 24.10): what each bot may
// see and do on the owner's desktop, and `desktop.*`. The bots' desktop
// tools are not played: grants come from `grant`, as the real daemon makes
// them when the owner allows a request.

import type { FakeBotloft, Handlers } from "./fake";
import { notFound } from "./fakeRules";
import type { BotId, DesktopGrant, DesktopLevel } from "./protocol.gen";

type DesktopMethods = Extract<keyof Handlers, `desktop.${string}`>;

export class FakeDesktop {
  readonly grants: DesktopGrant[] = [];

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
    };
  }
}
