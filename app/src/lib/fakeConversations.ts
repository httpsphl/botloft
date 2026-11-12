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
      createdAt: bot.createdAt,
      archivedAt: null,
    };
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
      "conversations.list": ({ botId }) => [this.main(botId), ...this.of(botId, false)],
      "conversations.archived": ({ botId }) => {
        this.fake.bot(botId, false);
        return this.of(botId, true);
      },
      "conversations.create": ({ botId }) => {
        const bot = this.fake.bot(botId);
        if (bot.agent !== "claude") {
          throw invalid(`agents that run on ${bot.agent} have only their main conversation`);
        }
        const now = Date.now();
        const made: Conversation = {
          id: `cnv_${crypto.randomUUID().replaceAll("-", "").slice(0, 26).toUpperCase()}`,
          botId,
          title: null,
          titleBy: null,
          main: false,
          state: "sleeping",
          generation: null,
          context: null,
          background: [],
          lastActivity: now,
          createdAt: now,
          archivedAt: null,
        };
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
