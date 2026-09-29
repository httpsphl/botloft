// The fake daemon's chats and approvals, with helpers for tests to play
// the bot: live text, replies, tool calls and permission requests.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, notFound } from "./fakeRules";
import type {
  Activity,
  ActivityKind,
  Approval,
  ApprovalItem,
  BotId,
  ChatBody,
  ChatItem,
  ToolItem,
} from "./protocol.gen";

/** The conversation-list line for an item, like the daemon's (spec 8.3). */
export function activityLine(body: ChatBody): { kind: ActivityKind; text: string } | null {
  const label = (name: string) => name.split("__").at(-1) ?? name;
  let kind: ActivityKind;
  let text: string;
  switch (body.kind) {
    case "inbound":
      kind = body.message.fromKind === "owner" ? "owner" : "message";
      text = body.message.body;
      break;
    case "reply":
      kind = "reply";
      text = body.text;
      break;
    case "tool":
      kind = "tool";
      text = body.summary ? `${label(body.name)} · ${body.summary}` : label(body.name);
      break;
    case "approval":
      kind = "approval";
      text = label(body.toolName);
      break;
    case "notice":
      kind = "notice";
      text = body.text;
      break;
    case "turn":
      return null;
  }
  const flat = text.split(/\s+/).filter(Boolean).join(" ");
  return { kind, text: flat.length <= 120 ? flat : `${flat.slice(0, 119).trimEnd()}…` };
}

export class FakeChat {
  readonly items: ChatItem[] = [];
  readonly approvals = new Map<string, Approval>();

  constructor(private readonly fake: FakeBotloft) {}

  /** Adds an item to the bot's chat and tells the app. */
  add(botId: BotId, body: ChatBody): ChatItem {
    const item: ChatItem = {
      id: this.fake.id("cht"),
      botId,
      body,
      createdAt: this.fake.now,
      updatedAt: this.fake.now,
    };
    this.items.push(item);
    this.announce(item, true);
    return item;
  }

  /** Text the bot is writing right now. */
  delta(botId: BotId, text: string): void {
    this.fake.emit({ name: "chat.delta", params: { botId, text } });
  }

  reply(botId: BotId, text: string): ChatItem {
    return this.add(botId, { kind: "reply", text });
  }

  /** A tool call in `running`, or with `changes` applied. */
  tool(botId: BotId, name: string, changes: Partial<ToolItem> = {}): ChatItem {
    return this.add(botId, {
      kind: "tool",
      toolUseId: this.fake.id("toolu"),
      name,
      summary: "",
      input: "{}",
      status: "running",
      output: null,
      ...changes,
    });
  }

  /** The tool call of `item` ends. */
  finish(item: ChatItem, output: string, failed = false): ChatItem {
    if (item.body.kind !== "tool") {
      throw new Error("not a tool call");
    }
    return this.update(item.id, { ...item.body, status: failed ? "failed" : "done", output });
  }

  turn(botId: BotId, error: string | null = null): ChatItem {
    return this.add(botId, { kind: "turn", durationMs: 4200, costUsd: 0.0123, error });
  }

  /** A permission request waiting for the owner. */
  ask(botId: BotId, toolName: string, summary: string, input = "{}"): ChatItem {
    const approval: Approval = {
      id: this.fake.id("apr"),
      botId,
      toolName,
      summary,
      input,
      status: "pending",
      note: null,
      createdAt: this.fake.now,
      answeredAt: null,
    };
    this.approvals.set(approval.id, approval);
    return this.add(botId, {
      kind: "approval",
      approvalId: approval.id,
      toolName,
      summary,
      input,
      status: "pending",
      note: null,
    });
  }

  handlers(): Pick<Handlers, "chat.history" | "approvals.answer"> {
    return {
      "chat.history": ({ botId, before, limit }) => {
        this.fake.bot(botId, false);
        const end = before ? this.items.findIndex((item) => item.id === before) : this.items.length;
        const older = this.items.slice(0, end < 0 ? this.items.length : end);
        return older
          .filter((item) => item.botId === botId)
          .reverse()
          .slice(0, limit ?? 50);
      },
      "approvals.answer": ({ approvalId, allow, note }) => {
        const approval = this.approvals.get(approvalId);
        if (!approval) {
          throw notFound(`approval ${approvalId}`);
        }
        if (approval.status !== "pending") {
          throw conflict("this request was already answered");
        }
        approval.status = allow ? "allowed" : "denied";
        approval.note = allow ? null : (note ?? null);
        approval.answeredAt = this.fake.now;
        // Claude Code leaves plan mode once the plan is approved.
        const bot = this.fake.bots.get(approval.botId);
        if (allow && approval.toolName === "ExitPlanMode" && bot?.permissionMode === "plan") {
          void this.fake.call("bots.setPermissionMode", { botId: bot.id, mode: "default" });
        }
        const item = this.items.find(
          (entry) => entry.body.kind === "approval" && entry.body.approvalId === approvalId,
        );
        if (item) {
          const body = item.body as { kind: "approval" } & ApprovalItem;
          this.update(item.id, { ...body, status: approval.status, note: approval.note });
        }
        return approval;
      },
    };
  }

  private update(id: string, body: ChatBody): ChatItem {
    const item = this.items.find((entry) => entry.id === id);
    if (!item) {
      throw notFound(`chat item ${id}`);
    }
    item.body = body;
    item.updatedAt = this.fake.now;
    this.announce(item, false);
    return item;
  }

  private announce(item: ChatItem, isNew: boolean): void {
    const line = isNew ? activityLine(item.body) : null;
    const activity: Activity | null = line === null ? null : { ...line, at: item.updatedAt };
    if (activity) {
      const bot = this.fake.bots.get(item.botId);
      if (bot) {
        bot.lastActivity = activity;
      }
    }
    this.fake.emit({ name: "chat.item", params: { item, activity } });
  }
}
