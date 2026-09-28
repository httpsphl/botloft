// The fake daemon's terminals: a buffer per bot, replay on attach, live
// output to attached bots, and the input and sizes the app sent.

import { decodeBytes, encodeBytes } from "./base64";
import type {
  BotId,
  TerminalAttachParams,
  TerminalAttachResult,
  TerminalWriteParams,
} from "./protocol.gen";
import type { ServerEvent } from "./rpc";

interface Buffer {
  generation: number;
  bytes: Uint8Array;
}

export class FakeTerminals {
  private readonly buffers = new Map<BotId, Buffer>();
  private readonly attached = new Set<BotId>();
  /** What the app typed into each terminal, decoded. */
  readonly input: { botId: BotId; text: string }[] = [];
  readonly sizes = new Map<BotId, { cols: number; rows: number }>();
  /** Every attach, with the point the app asked for. */
  readonly attaches: TerminalAttachParams[] = [];

  constructor(private readonly emit: (event: ServerEvent) => void) {}

  /** A new process: a new generation with an empty screen. */
  restart(botId: BotId, generation: number): void {
    this.buffers.set(botId, { generation, bytes: new Uint8Array() });
  }

  /** Output from the bot's process, streamed to an attached app. */
  output(botId: BotId, text: string): void {
    const buffer = this.buffer(botId);
    const chunk = new TextEncoder().encode(text);
    const offset = buffer.bytes.length;
    const bytes = new Uint8Array(offset + chunk.length);
    bytes.set(buffer.bytes);
    bytes.set(chunk, offset);
    buffer.bytes = bytes;
    if (this.attached.has(botId)) {
      this.data(botId, buffer.generation, offset, chunk);
    }
  }

  isAttached(botId: BotId): boolean {
    return this.attached.has(botId);
  }

  attach(params: TerminalAttachParams): TerminalAttachResult {
    this.attaches.push(params);
    const { botId, generation, offset } = params;
    const buffer = this.buffer(botId);
    const resumable =
      generation === buffer.generation && offset !== undefined && offset <= buffer.bytes.length;
    const from = resumable ? offset : 0;
    this.attached.add(botId);
    const replay = buffer.bytes.slice(from);
    if (replay.length > 0) {
      // The daemon answers the attach before the replay.
      setTimeout(() => this.data(botId, buffer.generation, from, replay), 0);
    }
    return {
      generation: buffer.generation,
      offset: from,
      reset: !resumable,
      liveOffset: buffer.bytes.length,
    };
  }

  detach(botId: BotId): void {
    this.attached.delete(botId);
  }

  write({ botId, data }: TerminalWriteParams): void {
    this.input.push({ botId, text: new TextDecoder().decode(decodeBytes(data)) });
  }

  private data(botId: BotId, generation: number, offset: number, bytes: Uint8Array): void {
    this.emit({
      name: "terminal.data",
      params: { botId, generation, offset, data: encodeBytes(bytes) },
    });
  }

  private buffer(botId: BotId): Buffer {
    let buffer = this.buffers.get(botId);
    if (!buffer) {
      buffer = { generation: 1, bytes: new Uint8Array() };
      this.buffers.set(botId, buffer);
    }
    return buffer;
  }
}
