// The fake daemon's Overview changes (spec 33.5): the real one reads git;
// here a test or the preview puts in what each agent's folders hold.

import type { Handlers } from "./fake";
import type { BotId, OverviewChanges } from "./protocol.gen";

export class FakeChanges {
  /** What `overview.changes` answers for each agent; nothing by default. */
  readonly byBot = new Map<BotId, OverviewChanges>();
  /** How many times each agent's changes were asked for. */
  readonly asked = new Map<BotId, number>();

  handlers(): Pick<Handlers, "overview.changes"> {
    return {
      "overview.changes": ({ botId }) => {
        this.asked.set(botId, (this.asked.get(botId) ?? 0) + 1);
        return this.byBot.get(botId) ?? { repos: [], events: [], prs: [], github: null };
      },
    };
  }
}
