// The fake daemon's reactions (spec 8.9): the owner's emoji on a bot's
// reply, waiting for the owner's next message, which takes them along.

import type { FakeBotloft, Handlers } from "./fake";
import { invalid, notFound } from "./fakeRules";
import type { BotId, MessageId, Reaction } from "./protocol.gen";
import { REACTIONS } from "./protocol.gen";

type ReactionMethods = Extract<keyof Handlers, `reactions.${string}`>;

export class FakeReactions {
  readonly reactions = new Map<string, Reaction>();

  constructor(private readonly fake: FakeBotloft) {}

  private changed(botId: BotId, itemId: string, reaction: Reaction | null) {
    this.fake.emit({ name: "reaction.changed", params: { botId, itemId, reaction } });
  }

  /** The owner's message `messageId` to `botId` takes what waits. */
  sendWith(botId: BotId, messageId: MessageId): void {
    for (const reaction of this.reactions.values()) {
      if (reaction.botId === botId && reaction.sentIn === null) {
        reaction.sentIn = messageId;
        this.changed(botId, reaction.itemId, { ...reaction });
      }
    }
  }

  handlers(): Pick<Handlers, ReactionMethods> {
    return {
      "reactions.list": ({ botId }) => {
        this.fake.bot(botId);
        return [...this.reactions.values()]
          .filter((reaction) => reaction.botId === botId)
          .map((reaction) => ({ ...reaction }));
      },
      "reactions.set": ({ botId, itemId, emoji }) => {
        if (emoji !== null && !(REACTIONS as readonly string[]).includes(emoji)) {
          throw invalid(`emoji must be one of ${REACTIONS.join(" ")}`);
        }
        this.fake.bot(botId);
        const item = this.fake.chat.items.find((one) => one.id === itemId && one.botId === botId);
        if (!item) {
          throw notFound(`chat item ${itemId}`);
        }
        if (item.body.kind !== "reply") {
          throw invalid("only a reply of the bot takes a reaction");
        }
        if (emoji === null) {
          if (this.reactions.delete(itemId)) {
            this.changed(botId, itemId, null);
          }
          return null;
        }
        const reaction: Reaction = {
          botId,
          itemId,
          emoji,
          createdAt: this.fake.now,
          sentIn: null,
        };
        this.reactions.set(itemId, reaction);
        this.changed(botId, itemId, { ...reaction });
        return { ...reaction };
      },
    };
  }
}
