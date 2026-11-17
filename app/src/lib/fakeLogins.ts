// The fake daemon's shared site logins (spec 35): the owner signs in to a
// site once in a window, and chooses which agents use the login. A test
// closes the window with `closeWindow`, saying whether a login was made.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, invalid, notFound } from "./fakeRules";
import type { LoginList, SiteLogin } from "./protocol.gen";

type LoginMethods = Extract<keyof Handlers, `logins.${string}`>;

/** The site of an address, as spec 21.5 has it. */
function siteOf(address: string): string | null {
  const text = address.trim();
  const url = text.includes("://") ? text : `https://${text}`;
  const match = /^https?:\/\/([^/?#:@]+)/i.exec(url);
  const host = match?.[1]?.toLowerCase().replace(/^www\./, "");
  return host || null;
}

export class FakeLogins {
  private logins: SiteLogin[] = [];
  private window: string | null = null;

  constructor(private readonly fake: FakeBotloft) {}

  list(): LoginList {
    return { logins: structuredClone(this.logins), window: this.window };
  }

  /** A login the owner already made, as if its window had closed. */
  add(site: string, botIds: string[] = [], signedAt = Date.now()): void {
    this.logins = [
      ...this.logins.filter((login) => login.site !== site),
      { site, signedAt, botIds },
    ];
  }

  /** The owner closes the window; `found` says whether they signed in. */
  closeWindow(found: boolean): void {
    const site = this.window;
    if (!site) {
      return;
    }
    this.window = null;
    if (found) {
      const had = this.logins.find((login) => login.site === site)?.botIds ?? [];
      this.add(site, had);
    }
    this.emit(site, found);
  }

  private emit(closed: string | null = null, found: boolean | null = null): void {
    this.fake.emit({ name: "logins.changed", params: { list: this.list(), closed, found } });
  }

  private find(site: string): SiteLogin {
    const login = this.logins.find((each) => each.site === site.trim().toLowerCase());
    if (!login) {
      throw notFound(`login ${site}`);
    }
    return login;
  }

  handlers(): Pick<Handlers, LoginMethods> {
    return {
      "logins.list": () => this.list(),
      "logins.open": ({ address }) => {
        const site = siteOf(address);
        if (!site) {
          throw invalid(`not a web address: ${JSON.stringify(address)}`);
        }
        if (this.window) {
          throw conflict("a login window is open");
        }
        this.window = site;
        this.emit();
        return { site };
      },
      "logins.share": ({ site, botIds }) => {
        const login = this.find(site);
        for (const id of botIds) {
          this.fake.bot(id);
        }
        login.botIds = [...new Set(botIds)];
        this.emit();
        return structuredClone(login);
      },
      "logins.remove": ({ site }) => {
        const login = this.find(site);
        this.logins = this.logins.filter((each) => each !== login);
        this.emit();
        return this.list();
      },
    };
  }
}
