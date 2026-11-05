// The fake daemon's side of the owner's own Chrome (spec 37): which agents
// the owner gave it to, and which one acts in it now.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict } from "./fakeRules";
import type { BotChrome, BotId, ChromeGrant, ChromeTurn } from "./protocol.gen";

export class FakeChrome {
  readonly grants: ChromeGrant[] = [];
  /** The agent acting in the Chrome now. */
  holder: BotId | null = null;

  constructor(private readonly fake: FakeBotloft) {}

  private of(botId: BotId): BotChrome {
    const grant = this.grants.find((have) => have.botId === botId);
    return { botId, givenAt: grant?.givenAt ?? null };
  }

  /** An agent takes the Chrome's turn, as the daemon announces it. */
  take(botId: BotId | null): void {
    this.holder = botId;
    this.fake.emit({ name: "chrome.turn", params: this.turn() });
  }

  private turn(): ChromeTurn {
    return { botId: this.holder };
  }

  handlers(): Pick<Handlers, "chrome.set" | "chrome.status" | "chrome.stop"> {
    return {
      "chrome.set": ({ botId, on, acceptedRisks }) => {
        const bot = this.fake.bot(botId);
        const at = this.grants.findIndex((have) => have.botId === botId);
        if (on) {
          if (bot.agent !== "claude") {
            throw conflict("only agents on Claude Code can use the owner's Chrome");
          }
          if (!acceptedRisks) {
            throw conflict("the owner has to accept the risks first");
          }
          if (at < 0) {
            this.grants.push({ botId, givenAt: this.fake.now });
          }
        } else if (at >= 0) {
          this.grants.splice(at, 1);
          if (this.holder === botId) {
            this.take(null);
          }
        }
        const chrome = this.of(botId);
        this.fake.emit({ name: "bot.chrome", params: chrome });
        return chrome;
      },
      "chrome.status": () => this.turn(),
      "chrome.stop": () => {
        if (this.holder !== null) {
          this.take(null);
        }
        return this.turn();
      },
    };
  }
}
