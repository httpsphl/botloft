// The fake daemon's permissions page (spec 15.1): every bot's grants,
// sites, rules and crew access in one call, and the sites of the bots'
// browsers, which the owner can take back (spec 21.5).

import type { FakeBotloft, Handlers } from "./fake";
import { notFound } from "./fakeRules";
import type { BotId, BrowserSite } from "./protocol.gen";

export class FakeSites {
  readonly sites: BrowserSite[] = [];

  constructor(private readonly fake: FakeBotloft) {}

  /** The owner allowed `site` for good, as the daemon keeps it. */
  allow(botId: BotId, site: string): void {
    if (!this.sites.some((known) => known.botId === botId && known.site === site)) {
      this.sites.push({ botId, site, allowedAt: this.fake.now });
    }
  }

  private of(botId: BotId): BrowserSite[] {
    return this.sites.filter((site) => site.botId === botId);
  }

  handlers(): Pick<Handlers, "permissions.list" | "sites.revoke"> {
    return {
      "permissions.list": () => {
        const live = (botId: BotId) => this.fake.bots.get(botId)?.archivedAt === null;
        return {
          desktop: this.fake.desktop.grants.filter((grant) => live(grant.botId)),
          sites: this.sites.filter((site) => live(site.botId)),
          rules: this.fake.allow.rules.filter((rule) => live(rule.botId)),
          crewAccess: this.fake.crewAccess.all().filter((access) => live(access.botId)),
        };
      },
      "sites.revoke": ({ botId, site }) => {
        this.fake.bot(botId);
        const at = this.sites.findIndex((known) => known.botId === botId && known.site === site);
        if (at < 0) {
          throw notFound(`site ${site}`);
        }
        this.sites.splice(at, 1);
        const left = { botId, sites: this.of(botId) };
        this.fake.emit({ name: "bot.sites", params: left });
        return left;
      },
    };
  }
}
