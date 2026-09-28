// The fake daemon's terminal, messages, deliveries and tasks.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, invalid, notFound } from "./fakeRules";
import type { Delivery, Message } from "./protocol.gen";

type Conversation = Pick<
  Handlers,
  | "terminal.attach"
  | "terminal.detach"
  | "terminal.write"
  | "terminal.resize"
  | "messages.send"
  | "messages.list"
  | "deliveries.list"
  | "deliveries.retry"
  | "tasks.list"
>;

export function conversationHandlers(fake: FakeBotloft): Conversation {
  return {
    "terminal.attach": ({ botId }) => {
      const bot = fake.bot(botId, false);
      return { generation: bot.generation ?? 1, offset: 0, reset: true };
    },
    "terminal.detach": () => null,
    "terminal.write": ({ botId, data }) => {
      fake.terminalInput.push({ botId, data });
      return null;
    },
    "terminal.resize": () => null,
    "messages.send": ({ botId, body }) => {
      const bot = fake.bot(botId);
      if (!body.trim()) {
        throw invalid("body must not be empty");
      }
      const message: Message = {
        id: fake.id("msg"),
        crewId: bot.crewId,
        fromKind: "owner",
        fromBotId: null,
        toBotId: botId,
        kind: "note",
        body: body.trim(),
        taskId: null,
        createdAt: fake.now,
      };
      fake.messages.push(message);
      const delivery: Delivery = {
        id: fake.id("dlv"),
        messageId: message.id,
        botId,
        state: "pending",
        attempts: 0,
        nextAttemptAt: fake.now,
        lastError: null,
        updatedAt: fake.now,
      };
      fake.deliveries.set(delivery.id, delivery);
      fake.emit({ name: "message.created", params: message });
      fake.emit({ name: "delivery.changed", params: delivery });
      return message;
    },
    "messages.list": ({ crewId, botId, before, limit }) => {
      const matches = fake.messages.filter(
        (message) =>
          (crewId === undefined || message.crewId === crewId) &&
          (botId === undefined || message.toBotId === botId || message.fromBotId === botId) &&
          (before === undefined || message.id < before),
      );
      return matches.reverse().slice(0, limit ?? 50);
    },
    "deliveries.list": ({ state, botId }) =>
      [...fake.deliveries.values()]
        .filter(
          (delivery) =>
            (state === undefined || delivery.state === state) &&
            (botId === undefined || delivery.botId === botId),
        )
        .sort((a, b) => b.updatedAt - a.updatedAt),
    "deliveries.retry": ({ deliveryId }) => {
      const delivery = fake.deliveries.get(deliveryId);
      if (!delivery) {
        throw notFound(`delivery ${deliveryId}`);
      }
      if (delivery.state !== "dead") {
        throw conflict("only a dead delivery can be retried");
      }
      Object.assign(delivery, { state: "pending", attempts: 0, lastError: null });
      fake.emit({ name: "delivery.changed", params: delivery });
      return delivery;
    },
    "tasks.list": ({ crewId, status }) =>
      [...fake.tasks.values()]
        .filter(
          (task) =>
            (crewId === undefined || task.crewId === crewId) &&
            (status === undefined || task.status === status),
        )
        .reverse(),
  };
}
