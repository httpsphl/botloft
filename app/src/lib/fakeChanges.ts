// The fake daemon's Overview changes (spec 33.5) and claude.ai connectors
// (spec 34): the real one reads git and asks Claude Code; here a test or
// the preview puts in what each agent's folders hold and what the account has.

import type { Handlers } from "./fake";
import type { BotId, ConnectorList, OverviewChanges } from "./protocol.gen";

export class FakeChanges {
  /** What `overview.changes` answers for each agent; nothing by default. */
  readonly byBot = new Map<BotId, OverviewChanges>();
  /** How many times each agent's changes were asked for. */
  readonly asked = new Map<BotId, number>();

  /** What `connectors.list` answers: the owner's claude.ai connectors. */
  connectors: ConnectorList = { connectors: [], checkedAt: 0 };

  handlers(): Pick<Handlers, "overview.changes" | "connectors.list"> {
    return {
      "connectors.list": () => this.connectors,
      "overview.changes": ({ botId }) => {
        this.asked.set(botId, (this.asked.get(botId) ?? 0) + 1);
        return this.byBot.get(botId) ?? { repos: [], events: [], prs: [], github: null };
      },
    };
  }
}
