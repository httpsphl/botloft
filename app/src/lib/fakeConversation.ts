// The fake daemon's messages, deliveries and tasks, with helpers for tests
// to play the bots' and the courier's part.

import { decodedSize } from "./base64";
import type { FakeBotloft, Handlers } from "./fake";
import { conflict, invalid, notFound } from "./fakeRules";
import {
  type Attachment,
  type AttachmentData,
  type AttachmentUpload,
  type BotId,
  type Delivery,
  type DeliveryState,
  FIELD_LIMITS,
  type Message,
  type MessageKind,
  type Task,
  type TaskId,
} from "./protocol.gen";

type Conversation = Pick<
  Handlers,
  | "messages.send"
  | "messages.list"
  | "attachments.read"
  | "deliveries.list"
  | "deliveries.retry"
  | "tasks.list"
>;

export class FakeConversation {
  readonly messages: Message[] = [];
  readonly deliveries = new Map<string, Delivery>();
  readonly tasks = new Map<TaskId, Task>();
  /** What was uploaded, by attachment id; `attachments.read` gives it back. */
  readonly files = new Map<string, AttachmentData>();

  constructor(private readonly fake: FakeBotloft) {}

  /** A message from a bot (or the daemon, without `from`) and its delivery. */
  say(options: { from?: BotId; to: BotId; body: string; kind?: MessageKind; taskId?: TaskId }): {
    message: Message;
    delivery: Delivery;
  } {
    const to = this.fake.bot(options.to);
    const message: Message = {
      id: this.fake.id("msg"),
      crewId: to.crewId,
      fromKind: options.from ? "bot" : "system",
      fromBotId: options.from ?? null,
      toBotId: to.id,
      kind: options.kind ?? (options.from ? "note" : "system"),
      body: options.body,
      taskId: options.taskId ?? null,
      routineId: null,
      attachments: [],
      createdAt: this.fake.now,
    };
    return { message, delivery: this.record(message) };
  }

  /** The courier moves a delivery along. */
  deliver(deliveryId: string, state: DeliveryState, lastError: string | null = null): Delivery {
    const delivery = this.delivery(deliveryId);
    const attempts = state === "dead" || lastError ? delivery.attempts + 1 : delivery.attempts;
    Object.assign(delivery, { state, lastError, attempts, updatedAt: this.fake.now });
    this.fake.emit({ name: "delivery.changed", params: delivery });
    return delivery;
  }

  /** The bot began the turn for the message: sent and read. */
  read(deliveryId: string): Delivery {
    const delivery = this.delivery(deliveryId);
    Object.assign(delivery, { state: "sent", readAt: this.fake.now, updatedAt: this.fake.now });
    this.fake.emit({ name: "delivery.changed", params: delivery });
    return delivery;
  }

  /** The delivery of `messageId`. */
  deliveryOf(messageId: string): Delivery {
    const delivery = [...this.deliveries.values()].find((d) => d.messageId === messageId);
    if (!delivery) {
      throw notFound(`delivery of ${messageId}`);
    }
    return delivery;
  }

  /** A task between two bots; `changes` override the defaults. */
  task(requester: BotId, assignee: BotId, changes: Partial<Task> = {}): Task {
    const existing = changes.id ? this.tasks.get(changes.id) : undefined;
    const task: Task = {
      id: this.fake.id("tsk"),
      crewId: this.fake.bot(assignee).crewId,
      requesterBotId: requester,
      assigneeBotId: assignee,
      status: "open",
      deadlineAt: this.fake.now + 2 * 60 * 60 * 1000,
      hops: 1,
      originTaskId: null,
      result: null,
      createdAt: this.fake.now,
      updatedAt: this.fake.now,
      ...existing,
      ...changes,
    };
    this.tasks.set(task.id, task);
    this.fake.emit({ name: "task.changed", params: task });
    return task;
  }

  handlers(): Conversation {
    return {
      "messages.send": ({ botId, body, attachments }) => {
        const bot = this.fake.bot(botId);
        const files = attachments ?? [];
        if (!body.trim() && files.length === 0) {
          throw invalid("body must not be empty");
        }
        if (files.length > FIELD_LIMITS.attachments) {
          throw invalid(`at most ${FIELD_LIMITS.attachments} files per message`);
        }
        const message: Message = {
          id: this.fake.id("msg"),
          crewId: bot.crewId,
          fromKind: "owner",
          fromBotId: null,
          toBotId: botId,
          kind: "note",
          body: body.trim(),
          taskId: null,
          routineId: null,
          attachments: files.map((file) => this.saved(file)),
          createdAt: this.fake.now,
        };
        this.record(message);
        return message;
      },
      "messages.list": ({ crewId, botId, before, limit }) => {
        const matches = this.messages.filter(
          (message) =>
            (crewId === undefined || message.crewId === crewId) &&
            (botId === undefined || message.toBotId === botId || message.fromBotId === botId) &&
            (before === undefined || message.id < before),
        );
        return matches.reverse().slice(0, limit ?? 50);
      },
      "attachments.read": ({ attachmentId }) => {
        const file = this.files.get(attachmentId);
        if (!file) {
          throw notFound(`attachment ${attachmentId}`);
        }
        return file;
      },
      "deliveries.list": ({ state, botId }) =>
        [...this.deliveries.values()]
          .filter(
            (delivery) =>
              (state === undefined || delivery.state === state) &&
              (botId === undefined || delivery.botId === botId),
          )
          .sort((a, b) => b.updatedAt - a.updatedAt),
      "deliveries.retry": ({ deliveryId }) => {
        const delivery = this.delivery(deliveryId);
        if (delivery.state !== "dead") {
          throw conflict("only a dead delivery can be retried");
        }
        Object.assign(delivery, { state: "pending", attempts: 0, lastError: null });
        this.fake.emit({ name: "delivery.changed", params: delivery });
        return delivery;
      },
      "tasks.list": ({ crewId, status }) =>
        [...this.tasks.values()]
          .filter(
            (task) =>
              (crewId === undefined || task.crewId === crewId) &&
              (status === undefined || task.status === status),
          )
          .reverse(),
    };
  }

  private record(message: Message): Delivery {
    this.messages.push(message);
    const delivery: Delivery = {
      id: this.fake.id("dlv"),
      messageId: message.id,
      botId: message.toBotId,
      state: "pending",
      attempts: 0,
      nextAttemptAt: this.fake.now,
      lastError: null,
      readAt: null,
      updatedAt: this.fake.now,
    };
    this.deliveries.set(delivery.id, delivery);
    this.fake.emit({ name: "message.created", params: message });
    this.fake.emit({ name: "delivery.changed", params: delivery });
    this.fake.chat.add(message.toBotId, { kind: "inbound", message });
    return delivery;
  }

  /** Where the daemon would save an uploaded file. */
  private saved(file: AttachmentUpload): Attachment {
    const mediaType = file.mediaType || "application/octet-stream";
    const attachment: Attachment = {
      id: this.fake.id("att"),
      name: file.name,
      mediaType,
      size: decodedSize(file.data),
      path: `attachments/2026-09-28/${file.name}`,
    };
    this.files.set(attachment.id, { mediaType, data: file.data });
    return attachment;
  }

  private delivery(deliveryId: string): Delivery {
    const delivery = this.deliveries.get(deliveryId);
    if (!delivery) {
      throw notFound(`delivery ${deliveryId}`);
    }
    return delivery;
  }
}
