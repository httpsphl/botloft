// The fake daemon's conversations of each bot (spec 40.5): the main one,
// which follows the bot, and the others the owner opens.

import { mainConversationOf } from "./conversations";
import type { FakeBotloft, Handlers } from "./fake";
import { invalid, notFound } from "./fakeRules";
import type { BotId, Conversation, ConversationId } from "./protocol.gen";

type ConversationMethods = Extract<keyof Handlers, `conversations.${string}`>;

export class FakeConversations {
  private readonly others = new Map<ConversationId, Conversation>();

  constructor(private readonly fake: FakeBotloft) {}

  /** The main conversation of a bot, as the daemon shows it. */
  main(botId: BotId): Conversation {
    const bot = this.fake.bot(botId, false);
    return {
      id: mainConversationOf(botId),
      botId,
      title: null,
      titleBy: null,
      main: true,
      state: bot.state,
      generation: bot.generation,
      context: bot.context,
      background: bot.background,
      lastActivity: bot.lastReplyAt ?? bot.createdAt,
      lastReplyAt: bot.lastReplyAt,
      readThrough: bot.readThrough,
      createdAt: bot.createdAt,
      archivedAt: null,
    };
  }

  /** The main conversation follows its bot: the app hears of it as the daemon tells it. */
  mainChanged(botId: BotId): void {
    this.fake.emit({ name: "conversation.changed", params: this.main(botId) });
  }

  /** Another conversation of `botId`, as a test opens it. */
  add(botId: BotId, title: string | null = null): Conversation {
    const made = this.make(botId);
    made.title = title;
    made.titleBy = title === null ? null : "owner";
    this.others.set(made.id, made);
    return structuredClone(made);
  }

  /** A conversation of the fake's, the main ones included. */
  get(conversationId: ConversationId): Conversation | null {
    const found = this.others.get(conversationId);
    if (found) {
      return structuredClone(found);
    }
    const botId = `bot_${conversationId.slice("cnv_".length)}`;
    return this.fake.bots.has(botId) ? this.main(botId) : null;
  }

  /** What a test makes another conversation do. */
  set(conversationId: ConversationId, change: Partial<Conversation>): void {
    const found = this.others.get(conversationId);
    if (!found) {
      throw notFound(`conversation ${conversationId}`);
    }
    Object.assign(found, change);
    this.changed(found);
  }

  private make(botId: BotId): Conversation {
    const now = Date.now();
    return {
      id: this.fake.id("cnv"),
      botId,
      title: null,
      titleBy: null,
      main: false,
      state: "sleeping",
      generation: null,
      context: null,
      background: [],
      lastActivity: now,
      lastReplyAt: null,
      readThrough: null,
      createdAt: now,
      archivedAt: null,
    };
  }

  /** An item went into another conversation's chat: its times move, quietly, as the daemon's do. */
  touched(conversationId: ConversationId, at: number, reply: boolean): void {
    const found = this.others.get(conversationId);
    if (found) {
      found.lastActivity = at;
      if (reply) {
        found.lastReplyAt = at;
      }
    }
  }

  /** The owner read another conversation (`chat.read`). */
  read(conversationId: ConversationId): void {
    const found = this.others.get(conversationId);
    if (found?.lastReplyAt) {
      found.readThrough = Math.max(found.readThrough ?? 0, found.lastReplyAt);
      this.changed(found);
    }
  }

  private of(botId: BotId, archived: boolean): Conversation[] {
    const mine = [...this.others.values()].filter(
      (each) => each.botId === botId && (each.archivedAt !== null) === archived,
    );
    return structuredClone(mine);
  }

  private other(conversationId: ConversationId, done: string): Conversation {
    const found = this.others.get(conversationId);
    if (!found) {
      if (this.fake.bots.has(`bot_${conversationId.slice("cnv_".length)}`)) {
        throw invalid(`the main conversation cannot be ${done}`);
      }
      throw notFound(`conversation ${conversationId}`);
    }
    return found;
  }

  private changed(conversation: Conversation): Conversation {
    this.fake.emit({ name: "conversation.changed", params: structuredClone(conversation) });
    return structuredClone(conversation);
  }

  handlers(): Pick<Handlers, ConversationMethods> {
    return {
      "conversations.list": ({ botId }) => {
        const bots = botId
          ? [botId]
          : [...this.fake.bots.values()]
              .filter((bot) => bot.archivedAt === null)
              .map((bot) => bot.id);
        return bots.flatMap((each) => [this.main(each), ...this.of(each, false)]);
      },
      "conversations.archived": ({ botId }) => {
        this.fake.bot(botId, false);
        return this.of(botId, true);
      },
      "conversations.create": ({ botId }) => {
        const bot = this.fake.bot(botId);
        if (bot.agent !== "claude") {
          throw invalid(`agents that run on ${bot.agent} have only their main conversation`);
        }
        const made = this.make(botId);
        this.others.set(made.id, made);
        return this.changed(made);
      },
      "conversations.rename": ({ conversationId, title }) => {
        const found = this.other(conversationId, "renamed");
        const named = title.trim();
        if (named === "") {
          throw invalid("title must not be empty");
        }
        found.title = named;
        found.titleBy = "owner";
        return this.changed(found);
      },
      "conversations.archive": ({ conversationId }) => {
        const found = this.other(conversationId, "archived");
        found.archivedAt ??= Date.now();
        found.state = "archived";
        return this.changed(found);
      },
      "conversations.unarchive": ({ conversationId }) => {
        const found = this.other(conversationId, "archived");
        found.archivedAt = null;
        found.state = "sleeping";
        return this.changed(found);
      },
    };
  }
}
