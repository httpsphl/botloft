// One bot's terminal stream (spec 8): attach, replay, live output, input
// and size. Independent of xterm.js so it can be tested with the fake.
//
// The client keeps the generation and the offset it has written up to.
// Output of a newer generation is a new screen. Output past that offset
// means something was missed, so it attaches again from what it has; the
// daemon then replays only the gap, or the whole buffer with `reset`.

import type { BotloftApi } from "../../lib/api";
import { decodeBytes, encodeBinary, encodeText } from "../../lib/base64";
import type { BotId, TerminalData } from "../../lib/protocol.gen";

/** Where the output goes. */
export interface Screen {
  /**
   * `history` marks replayed output: the terminal queries in it were asked
   * long ago, so the screen must not answer them.
   */
  write(data: Uint8Array, history: boolean): void;
  /** Clears everything, as for a new process. */
  reset(): void;
}

export class TerminalSession {
  private generation: number | null = null;
  /** Bytes of `generation` already written to the screen. */
  private offset = 0;
  /** Where the last attach's replay ends and live output begins. */
  private liveFrom = 0;
  private attaching = false;
  /** Output that arrived while an attach was in flight. */
  private held: TerminalData[] = [];
  private size: { cols: number; rows: number } | null = null;
  private stopped = false;

  constructor(
    private readonly api: BotloftApi,
    private readonly botId: BotId,
    private readonly screen: Screen,
  ) {}

  /** Starts streaming; the returned function detaches. */
  start(): () => void {
    const offData = this.api.subscribe((event) => {
      if (event.name === "terminal.data" && event.params.botId === this.botId) {
        this.receive(event.params);
      }
    });
    let open = this.api.connection().kind === "open";
    // A new connection has no stream yet: attach again from what we have.
    const offConnection = this.api.onConnection((state) => {
      const nowOpen = state.kind === "open";
      if (nowOpen && !open) {
        void this.attach();
      }
      open = nowOpen;
    });
    if (open) {
      void this.attach();
    }
    return () => {
      this.stopped = true;
      offData();
      offConnection();
      if (this.api.connection().kind === "open") {
        this.api.call("terminal.detach", { botId: this.botId }).catch(() => {});
      }
    };
  }

  /** Keys and pastes from the owner. Dropped while disconnected. */
  input(text: string): void {
    this.send(encodeText(text));
  }

  inputBinary(binary: string): void {
    this.send(encodeBinary(binary));
  }

  resize(cols: number, rows: number): void {
    if (this.size?.cols === cols && this.size.rows === rows) {
      return;
    }
    this.size = { cols, rows };
    this.api.call("terminal.resize", { botId: this.botId, cols, rows }).catch(() => {});
  }

  private send(data: string): void {
    if (this.stopped || this.api.connection().kind !== "open") {
      return;
    }
    // A bot without a process refuses input; its screen already says so.
    this.api.call("terminal.write", { botId: this.botId, data }).catch(() => {});
  }

  private async attach(): Promise<void> {
    if (this.attaching || this.stopped) {
      return;
    }
    this.attaching = true;
    const from =
      this.generation === null ? {} : { generation: this.generation, offset: this.offset };
    try {
      const point = await this.api.call("terminal.attach", { botId: this.botId, ...from });
      if (this.stopped) {
        return;
      }
      if (point.reset) {
        this.screen.reset();
      }
      this.generation = point.generation;
      this.offset = point.offset;
      this.liveFrom = point.liveOffset;
    } catch {
      // A lost connection attaches again when it comes back.
    } finally {
      this.attaching = false;
      for (const chunk of this.held.splice(0)) {
        this.receive(chunk);
      }
    }
  }

  private receive(chunk: TerminalData): void {
    if (this.stopped) {
      return;
    }
    if (this.attaching) {
      this.held.push(chunk);
      return;
    }
    if (chunk.generation !== this.generation) {
      // Generations only grow; an older one is a stream that ended.
      if (this.generation !== null && chunk.generation < this.generation) {
        return;
      }
      this.screen.reset();
      this.generation = chunk.generation;
      this.offset = 0;
      this.liveFrom = 0;
    }
    const bytes = decodeBytes(chunk.data);
    const end = chunk.offset + bytes.length;
    if (end <= this.offset) {
      return;
    }
    if (chunk.offset > this.offset) {
      void this.attach();
      return;
    }
    const fresh = bytes.subarray(this.offset - chunk.offset);
    const history = Math.min(fresh.length, Math.max(0, this.liveFrom - this.offset));
    if (history > 0) {
      this.screen.write(fresh.subarray(0, history), true);
    }
    if (history < fresh.length) {
      this.screen.write(fresh.subarray(history), false);
    }
    this.offset = end;
  }
}
