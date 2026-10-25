// The fake daemon's agent keys (spec 32): the names and descriptions it
// lists, and the values only to tell which ones are set. A value is never
// handed back.

import type { FakeBotloft, Handlers } from "./fake";
import type { AgentKey, BotId } from "./protocol.gen";

export class FakeKeys {
  /** Each agent's keys by name; the value is kept only for tests to check. */
  readonly saved = new Map<BotId, Map<string, AgentKey & { value: string }>>();

  constructor(private readonly fake: FakeBotloft) {}

  private list(botId: BotId): AgentKey[] {
    this.fake.bot(botId);
    const keys = [...(this.saved.get(botId)?.values() ?? [])];
    return keys
      .sort((a, b) => a.name.localeCompare(b.name))
      .map(({ name, description, setAt, missing }) => ({ name, description, setAt, missing }));
  }

  handlers(): Pick<Handlers, "keys.list" | "keys.set" | "keys.delete"> {
    return {
      "keys.list": ({ botId }) => this.list(botId),
      "keys.set": ({ botId, name, value, description }) => {
        this.fake.bot(botId);
        const keys = this.saved.get(botId) ?? new Map();
        const before = keys.get(name);
        keys.set(name, {
          name,
          value: value.trim(),
          description: description ?? before?.description ?? "",
          setAt: this.fake.now,
          missing: false,
        });
        this.saved.set(botId, keys);
        return this.list(botId);
      },
      "keys.delete": ({ botId, name }) => {
        this.saved.get(botId)?.delete(name);
        return this.list(botId);
      },
    };
  }
}
