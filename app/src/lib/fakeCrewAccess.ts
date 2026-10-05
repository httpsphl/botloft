// The fake daemon's lasting access to other crews (spec 10.4): what each
// bot may reach for good, and `crewAccess.*`. The request itself is a plain
// approval (`chat.ask` with the tool's name).

import type { FakeBotloft, Handlers } from "./fake";
import { notFound } from "./fakeRules";
import type { BotId, CrewAccess } from "./protocol.gen";

type AccessMethods = Extract<keyof Handlers, `crewAccess.${string}`>;

export class FakeCrewAccess {
  private list: CrewAccess[] = [];
  private next = 1;

  constructor(private readonly fake: FakeBotloft) {}

  /** `bot` may reach `crewName` for good, or only `target` in it. */
  add(
    bot: BotId,
    crewName: string,
    target?: { id: BotId; name: string },
    kinds: { talk?: boolean; read?: boolean; edit?: boolean } = { talk: true },
  ): CrewAccess {
    const access: CrewAccess = {
      id: `cxa_${this.next++}`,
      botId: bot,
      crewId: `crw_${crewName.toLowerCase()}`,
      crewName,
      targetBotId: target?.id ?? null,
      targetName: target?.name ?? null,
      talk: kinds.talk ?? false,
      read: (kinds.read ?? false) || (kinds.edit ?? false),
      edit: kinds.edit ?? false,
      createdAt: this.fake.now,
    };
    this.list.push(access);
    return access;
  }

  handlers(): Pick<Handlers, AccessMethods> {
    const of = (bot: BotId) => this.list.filter((access) => access.botId === bot);
    return {
      "crewAccess.list": ({ botId }) => {
        this.fake.bot(botId, false);
        return of(botId);
      },
      "crewAccess.revoke": ({ accessId }) => {
        const found = this.list.find((access) => access.id === accessId);
        if (!found) {
          throw notFound(`access ${accessId}`);
        }
        this.list = this.list.filter((access) => access.id !== accessId);
        return of(found.botId);
      },
    };
  }
}
