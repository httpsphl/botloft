// The conversations on the phone (spec 28.12): the list, the open chat, the
// messages the owner writes. All of it in memory; the only thing kept on the
// device is one number per bot (the last reply seen), never a text.

import type {
  BotState,
  ChatLine,
  FromPhone,
  PhoneFile,
  PhoneItem,
  ToPhone,
} from "../lib/protocol.gen";
import type { PhoneState } from "./client";
import { FILES_MAX, type Outgoing, UploadError } from "./outgoing";

/** A message the owner wrote that the computer has not shown back yet. */
export interface Pending {
  clientId: string;
  text: string;
  /** The files that go with it, kept here to try again. */
  files?: Outgoing[];
  /** `sent`: the computer took it; it shows when the bot gets it. */
  status: "sending" | "sent" | "failed";
  reason?: string;
}

/** Leaves a file on the server for the computer (spec 28.14). */
export type Upload = (file: Outgoing) => Promise<PhoneFile>;

/** How much longer the computer may take for each MB it brings down. */
const WAIT_PER_MB = 5000;

/** The open conversation. */
export interface Convo {
  botId: string;
  /** Oldest first. */
  items: PhoneItem[];
  /** There are older items than these. */
  more: boolean;
  loaded: boolean;
  /** Older items are on their way. */
  older: boolean;
  /** The reply the bot is writing now, whole. */
  live: string;
  working: boolean;
  pending: Pending[];
}

/** The part of the phone's state that is about conversations. */
export interface ChatPart {
  /** The list, the newest activity first. */
  chats: ChatLine[];
  chatsLoaded: boolean;
  convo: Convo | null;
  /** The last reply the owner has seen, by bot. */
  seen: Record<string, number>;
}

/** The number kept for each bot: the last reply the owner has seen. */
export interface SeenStore {
  /** `null` when nothing was kept yet. */
  load(): Record<string, number> | null;
  save(seen: Record<string, number>): void;
}

/** The browser's own storage: one number per bot, and nothing else. */
export function browserSeen(key = "botloft.phone.seen"): SeenStore {
  return {
    load: () => {
      try {
        const kept = localStorage.getItem(key);
        return kept === null ? null : (JSON.parse(kept) as Record<string, number>);
      } catch {
        return null;
      }
    },
    save: (seen) => {
      try {
        localStorage.setItem(key, JSON.stringify(seen));
      } catch {
        // Without it every reply looks new once; nothing is lost.
      }
    },
  };
}

export const noChats: ChatPart = { chats: [], chatsLoaded: false, convo: null, seen: {} };

/** Longest text the phone sends, in bytes: the sealed message holds 16 KiB. */
export const SEND_MAX = 12 * 1024;

const working = (state: BotState) => state === "busy" || state === "needs_approval";

const byActivity = (lines: ChatLine[]) =>
  [...lines].sort((a, b) => (b.last?.at ?? 0) - (a.last?.at ?? 0));

export class Talk {
  private req = 0;
  private current: { req: number; botId: string; older: boolean; buffer: PhoneItem[] } | null =
    null;
  private list: ChatLine[] = [];
  private counter = 0;
  private readonly timers = new Map<string, ReturnType<typeof setTimeout>>();

  constructor(
    private readonly get: () => PhoneState,
    private readonly set: (change: Partial<PhoneState>) => void,
    private readonly send: (message: FromPhone) => Promise<boolean>,
    private readonly store?: SeenStore,
    private readonly sentWait = 20000,
    private readonly upload?: Upload,
  ) {
    this.set({ seen: store?.load() ?? {} });
  }

  private convo(): Convo | null {
    return this.get().convo;
  }

  private change(change: Partial<Convo>): void {
    const convo = this.convo();
    if (convo) {
      this.set({ convo: { ...convo, ...change } });
    }
  }

  /** The connection is up (again): ask for the list, and the open chat. */
  async ask(): Promise<void> {
    await this.send({ t: "chats" });
    const convo = this.convo();
    if (convo) {
      await this.watch(convo.botId);
    }
  }

  private async watch(botId: string): Promise<void> {
    await this.send({ t: "watch", botId });
    this.req += 1;
    this.current = { req: this.req, botId, older: false, buffer: [] };
    await this.send({ t: "history", req: this.req, botId });
  }

  open = async (botId: string): Promise<void> => {
    this.set({
      convo: {
        botId,
        items: [],
        more: false,
        loaded: false,
        older: false,
        live: "",
        working: this.list.find((line) => line.botId === botId)?.state === "busy",
        pending: [],
      },
    });
    this.markSeen(botId);
    await this.watch(botId);
  };

  leave = async (): Promise<void> => {
    const convo = this.convo();
    if (convo) {
      this.markSeen(convo.botId);
    }
    this.current = null;
    this.set({ convo: null });
    await this.send({ t: "watch" });
  };

  older = async (): Promise<void> => {
    const convo = this.convo();
    const oldest = convo?.items[0];
    if (!convo || !oldest || convo.older || !convo.more) {
      return;
    }
    this.req += 1;
    this.current = { req: this.req, botId: convo.botId, older: true, buffer: [] };
    this.change({ older: true });
    const sent = await this.send({
      t: "history",
      req: this.req,
      botId: convo.botId,
      before: oldest.id,
    });
    if (!sent) {
      this.change({ older: false });
    }
  };

  /**
   * Writes to the open bot, with files if any; the message shows at once as
   * sending. The files go to the server first, and their keys with the text.
   */
  write = async (text: string, files: Outgoing[] = []): Promise<boolean> => {
    const convo = this.convo();
    const body = text.trim();
    if (
      !convo ||
      (body === "" && files.length === 0) ||
      files.length > FILES_MAX ||
      new TextEncoder().encode(body).length > SEND_MAX
    ) {
      return false;
    }
    this.counter += 1;
    const clientId = `${Date.now().toString(36)}-${this.counter}`;
    this.change({
      pending: [
        ...convo.pending.filter((pending) => pending.status !== "failed"),
        { clientId, text: body, status: "sending", ...(files.length > 0 ? { files } : {}) },
      ],
    });
    const left: PhoneFile[] = [];
    for (const file of files) {
      try {
        if (!this.upload) {
          throw new UploadError("failed");
        }
        left.push(await this.upload(file));
      } catch (failure) {
        this.settle(clientId, "failed", failure instanceof UploadError ? failure.reason : "failed");
        return false;
      }
    }
    const message: FromPhone = {
      t: "send",
      clientId,
      botId: convo.botId,
      text: body,
      ...(left.length > 0 ? { files: left } : {}),
    };
    const sent = await this.send(message).catch(() => false);
    if (!sent) {
      this.settle(clientId, "failed", "offline");
      return false;
    }
    // The computer brings the files down before it answers.
    const megabytes = Math.ceil(left.reduce((sum, file) => sum + file.size, 0) / (1 << 20));
    this.timers.set(
      clientId,
      setTimeout(
        () => this.settle(clientId, "failed", "timeout", "sending"),
        this.sentWait + megabytes * WAIT_PER_MB,
      ),
    );
    return true;
  };

  private settle(
    clientId: string,
    status: Pending["status"],
    reason?: string,
    only?: Pending["status"],
  ): void {
    this.stopWaiting(clientId);
    const convo = this.convo();
    if (!convo) {
      return;
    }
    this.change({
      pending: convo.pending.map((pending) =>
        pending.clientId === clientId && (!only || pending.status === only)
          ? { ...pending, status, ...(reason ? { reason } : {}) }
          : pending,
      ),
    });
  }

  private stopWaiting(clientId: string): void {
    const timer = this.timers.get(clientId);
    if (timer) {
      clearTimeout(timer);
      this.timers.delete(clientId);
    }
  }

  /** What the owner read on the computer counts as seen here too. */
  private absorb(read: Record<string, number>): void {
    const seen = { ...this.get().seen };
    let changed = false;
    for (const [botId, upto] of Object.entries(read)) {
      if (upto > (seen[botId] ?? 0)) {
        seen[botId] = upto;
        changed = true;
      }
    }
    if (changed) {
      this.set({ seen });
      this.store?.save(seen);
    }
  }

  private absorbLines(lines: ChatLine[]): void {
    this.absorb(
      Object.fromEntries(
        lines.filter((line) => line.readAt !== undefined).map((line) => [line.botId, line.readAt]),
      ) as Record<string, number>,
    );
  }

  private markSeen(botId: string): void {
    const at = this.list.find((line) => line.botId === botId)?.lastReplyAt;
    if (at === undefined || this.get().seen[botId] === at) {
      return;
    }
    const seen = { ...this.get().seen, [botId]: at };
    this.set({ seen });
    this.store?.save(seen);
  }

  /** Whether `message` was about conversations. */
  handle(message: ToPhone): boolean {
    switch (message.t) {
      case "chats":
        this.onChats(message.bots, message.first);
        return true;
      case "read":
        this.absorb({ [message.botId]: message.upto });
        return true;
      case "line": {
        const rest = this.list.filter((line) => line.botId !== message.bot.botId);
        this.list = byActivity([...rest, message.bot]);
        this.set({ chats: this.list });
        this.absorbLines([message.bot]);
        if (this.convo()?.botId === message.bot.botId) {
          this.markSeen(message.bot.botId);
        }
        return true;
      }
      case "history":
        this.onHistory(message);
        return true;
      case "item":
        this.onItem(message.botId, message.item);
        return true;
      case "live":
        if (this.convo()?.botId === message.botId) {
          this.change({ live: message.text });
        }
        return true;
      case "state":
        if (this.convo()?.botId === message.botId) {
          this.change({ working: working(message.state) });
        }
        return true;
      case "sent":
        this.settle(message.clientId, message.ok ? "sent" : "failed", message.reason);
        return true;
      default:
        return false;
    }
  }

  private onChats(bots: ChatLine[], first: boolean): void {
    this.list = byActivity(first ? bots : [...this.list, ...bots]);
    // A phone that never looked at the list starts with everything read.
    if (first && this.store && this.store.load() === null) {
      const start = Object.fromEntries(bots.map((bot) => [bot.botId, bot.lastReplyAt ?? 0]));
      this.set({ seen: start });
      this.store.save(start);
    }
    this.set({ chats: this.list, chatsLoaded: true });
    this.absorbLines(bots);
  }

  private onHistory(message: Extract<ToPhone, { t: "history" }>): void {
    const current = this.current;
    const convo = this.convo();
    if (!current || !convo || current.req !== message.req || current.botId !== message.botId) {
      return;
    }
    current.buffer.push(...message.items);
    if (!message.done) {
      return;
    }
    this.current = null;
    const page = current.buffer;
    const known = new Set(page.map((item) => item.id));
    let items: PhoneItem[];
    if (current.older) {
      items = [...page, ...convo.items.filter((item) => !known.has(item.id))];
    } else {
      // What came while the page was on its way is newer than the page.
      const start = page[0]?.at ?? 0;
      items = [...page, ...convo.items.filter((item) => !known.has(item.id) && item.at >= start)];
    }
    this.change({ items, more: message.more, loaded: true, older: false });
  }

  private onItem(botId: string, item: PhoneItem): void {
    const convo = this.convo();
    if (!convo || convo.botId !== botId) {
      return;
    }
    const at = convo.items.findIndex((other) => other.id === item.id);
    const items =
      at >= 0
        ? convo.items.map((other, index) => (index === at ? item : other))
        : [...convo.items, item];
    let pending = convo.pending;
    if (item.kind === "you") {
      // The computer may rename a file it already has, so only the count matters.
      const mine = pending.findIndex(
        (one) =>
          one.status !== "failed" &&
          one.text.trim() === item.text.trim() &&
          (one.files?.length ?? 0) === (item.files?.length ?? 0),
      );
      if (mine >= 0) {
        const gone = pending[mine];
        if (gone) {
          this.stopWaiting(gone.clientId);
        }
        pending = pending.filter((_, index) => index !== mine);
      }
    }
    this.change({ items, pending });
  }
}
