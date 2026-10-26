// The fake daemon's agent keys (spec 32): the names and descriptions it
// lists, and the values only to tell which ones are set. A value is never
// handed back. A bot's request for a key is a chat item, closed when the key
// is saved with its id or when the owner says not now.

import type { FakeBotloft, Handlers } from "./fake";
import type {
  AgentKey,
  BotId,
  ChatItem,
  KeyHit,
  KeyRequestItem,
  KeyRequestStatus,
} from "./protocol.gen";

export class FakeKeys {
  /** Each agent's keys by name; the value is kept only for tests to check. */
  readonly saved = new Map<BotId, Map<string, AgentKey & { value: string }>>();

  /**
   * What `keys.scan` finds (spec 32.7), with the value behind each hit; the
   * real daemon reads the conversations, here a test puts them in.
   */
  readonly pasted: (KeyHit & { value: string })[] = [];

  constructor(private readonly fake: FakeBotloft) {}

  private hits(): KeyHit[] {
    return this.pasted.map(({ value: _value, ...hit }) => hit);
  }

  /** The bot asks the owner for a key, as `ask_key` does. */
  ask(botId: BotId, name: string, why: string): ChatItem {
    return this.fake.chat.add(botId, { kind: "key_request", name, why, status: "open" });
  }

  /**
   * `text` with each of the bot's saved values, 8 characters or more and the
   * longest first, swapped for `[chave NAME]`, as the daemon does (spec 32.6).
   */
  redact(botId: BotId, text: string): string {
    const values = [...(this.saved.get(botId)?.values() ?? [])]
      .filter((key) => key.value.length >= 8)
      .sort((a, b) => b.value.length - a.value.length);
    return values.reduce((done, key) => done.replaceAll(key.value, `[chave ${key.name}]`), text);
  }

  private list(botId: BotId): AgentKey[] {
    this.fake.bot(botId);
    const keys = [...(this.saved.get(botId)?.values() ?? [])];
    return keys
      .sort((a, b) => a.name.localeCompare(b.name))
      .map(({ name, description, setAt, missing }) => ({ name, description, setAt, missing }));
  }

  /** The open request `id`, checked the way the daemon does. */
  private request(id: string, botId?: BotId, name?: string): ChatItem & { body: KeyRequestItem } {
    const item = this.fake.chat.items.find((entry) => entry.id === id);
    if (item?.body.kind !== "key_request" || (botId && item.botId !== botId)) {
      throw new Error(`request ${id} does not exist`);
    }
    if (item.body.status !== "open") {
      throw new Error(`request ${id} was already answered`);
    }
    if (name && item.body.name !== name) {
      throw new Error(`the request asks for ${item.body.name}, not ${name}`);
    }
    return item as ChatItem & { body: KeyRequestItem };
  }

  private close(item: ChatItem & { body: KeyRequestItem }, status: KeyRequestStatus): void {
    this.fake.chat.update(item.id, { ...item.body, kind: "key_request", status });
  }

  handlers(): Pick<
    Handlers,
    "keys.list" | "keys.set" | "keys.delete" | "keys.dismiss" | "keys.scan" | "keys.scrub"
  > {
    return {
      "keys.scan": () => this.hits(),
      "keys.scrub": ({ hits }) => {
        for (const { id, keep } of hits) {
          const at = this.pasted.findIndex((hit) => hit.id === id);
          const hit = this.pasted[at];
          if (!hit) {
            throw new Error(`hit ${id} is gone`);
          }
          if (keep) {
            this.handlers()["keys.set"]({ botId: keep.botId, name: keep.name, value: hit.value });
          }
          this.pasted.splice(at, 1);
        }
        return this.hits();
      },
      "keys.list": ({ botId }) => this.list(botId),
      "keys.set": ({ botId, name, value, description, requestId }) => {
        this.fake.bot(botId);
        const request = requestId ? this.request(requestId, botId, name) : null;
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
        if (request) {
          this.close(request, "saved");
        }
        return this.list(botId);
      },
      "keys.delete": ({ botId, name }) => {
        this.saved.get(botId)?.delete(name);
        return this.list(botId);
      },
      "keys.dismiss": ({ requestId }) => {
        this.close(this.request(requestId), "dismissed");
        return null;
      },
    };
  }
}
