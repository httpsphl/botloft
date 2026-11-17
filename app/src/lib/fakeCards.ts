// The fake daemon's agent cards (spec 8.11): `show` plays a bot calling
// `show_card`, and `chat.act` answers one version like the daemon does,
// the answer going to the bot as the owner's message on the card.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, invalid, notFound } from "./fakeRules";
import type { BotId, CardBlock, CardItem, ChatItem, ChatItemId } from "./protocol.gen";

export class FakeCards {
  constructor(private readonly fake: FakeBotloft) {}

  /** The bot shows a card, or `card` again as its next version. */
  show(botId: BotId, title: string, blocks: CardBlock[], card?: ChatItemId): ChatItem {
    if (card) {
      const shown = this.card(card);
      return this.fake.chat.update(card, {
        kind: "card",
        version: shown.version + 1,
        title,
        blocks,
        answer: null,
      });
    }
    return this.fake.chat.add(botId, { kind: "card", version: 1, title, blocks, answer: null });
  }

  private item(id: string): ChatItem {
    const item = this.fake.chat.items.find((entry) => entry.id === id);
    if (item?.body.kind !== "card") {
      throw notFound(`card ${id}`);
    }
    return item;
  }

  private card(id: string): CardItem {
    const { body } = this.item(id);
    if (body.kind !== "card") {
      throw notFound(`card ${id}`);
    }
    return body;
  }

  handlers(): Pick<Handlers, "chat.act"> {
    return {
      "chat.act": ({ itemId, version, button, form }) => {
        const item = this.item(itemId);
        const card = this.card(itemId);
        const bot = this.fake.bot(item.botId);
        if (version !== card.version) {
          throw conflict(`${bot.name} changed this card since you saw it; look at the new one`);
        }
        if (card.answer) {
          throw conflict("this card was already answered");
        }
        const text = button !== undefined ? pressed(card, button) : sent(card, form ?? {});
        this.fake.conversation.record({
          id: this.fake.id("msg"),
          crewId: bot.crewId,
          fromKind: "owner",
          fromBotId: null,
          toBotId: bot.id,
          kind: "note",
          body: text,
          taskId: null,
          routineId: null,
          questionId: null,
          attachments: [],
          replyTo: { itemId, text: card.title },
          createdAt: this.fake.now,
        });
        return this.fake.chat.update(itemId, {
          ...card,
          kind: "card",
          answer: { version, button: button ?? null, text, at: this.fake.now },
        });
      },
    };
  }
}

function pressed(card: CardItem, id: string): string {
  for (const block of card.blocks) {
    if (block.type === "buttons") {
      const found = block.buttons.find((button) => button.id === id);
      if (found) {
        return found.label;
      }
    }
  }
  throw invalid(`the card has no button "${id}"`);
}

function sent(card: CardItem, values: Record<string, string>): string {
  const form = card.blocks.find((block) => block.type === "form");
  if (form?.type !== "form") {
    throw invalid("the card has no form");
  }
  return form.fields
    .map((field) => {
      const value = (values[field.id] ?? "").trim();
      if (field.kind === "check") {
        return `${field.label}: ${value === "true" ? "✓" : "✗"}`;
      }
      if (!value && field.required) {
        throw invalid(`"${field.label}" must be filled in`);
      }
      return `${field.label}: ${value || "—"}`;
    })
    .join("\n");
}
