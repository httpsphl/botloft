import { describe, expect, test } from "vitest";
import { encodeText } from "../../lib/base64";
import { FakeBotloft } from "../../lib/fake";
import { type Screen, TerminalSession } from "./session";

/** A screen that keeps text, with a mark for every reset. */
class TextScreen implements Screen {
  text = "";
  /** The part of `text` written as replayed history. */
  history = "";
  resets = 0;
  private readonly decoder = new TextDecoder();

  write(data: Uint8Array, history: boolean): void {
    const text = this.decoder.decode(data, { stream: true });
    this.text += text;
    if (history) {
      this.history += text;
    }
  }

  reset(): void {
    this.resets += 1;
    this.text = "";
    this.history = "";
  }
}

/** Lets the fake's replay (sent after the attach answer) arrive. */
const settle = () => new Promise((resolve) => setTimeout(resolve, 5));

function setup() {
  const fake = new FakeBotloft();
  const crew = fake.addCrew("Ops");
  const bot = fake.addBot(crew.id, "Scout");
  const screen = new TextScreen();
  const session = new TerminalSession(fake, bot.id, screen);
  return { fake, bot, screen, session };
}

describe("terminal session", () => {
  test("replays the buffer on attach, then follows live output", async () => {
    const { fake, bot, screen, session } = setup();
    fake.terminals.output(bot.id, "Welcome to Claude Code\r\n");
    const stop = session.start();
    await settle();
    expect(screen.text).toBe("Welcome to Claude Code\r\n");
    expect(screen.resets).toBe(1);
    fake.terminals.output(bot.id, "> ");
    expect(screen.text).toBe("Welcome to Claude Code\r\n> ");
    // Only the replay is history: the queries in it must go unanswered.
    expect(screen.history).toBe("Welcome to Claude Code\r\n");
    stop();
    expect(fake.terminals.isAttached(bot.id)).toBe(false);
    fake.terminals.output(bot.id, "after");
    expect(screen.text).not.toContain("after");
  });

  test("a new generation is a new screen", async () => {
    const { fake, bot, screen, session } = setup();
    fake.terminals.output(bot.id, "old session");
    session.start();
    await settle();
    fake.setBotState(bot.id, "launching", 2);
    fake.terminals.output(bot.id, "new session");
    expect(screen.text).toBe("new session");
    expect(screen.resets).toBe(2);
    // A new process's output is live from its first byte.
    expect(screen.history).toBe("");
  });

  test("a gap in the output attaches again from what it has", async () => {
    const { fake, bot, screen, session } = setup();
    fake.terminals.output(bot.id, "abc");
    session.start();
    await settle();
    // Output the app missed (say, it fell behind), then more.
    fake.terminals.detach(bot.id);
    fake.terminals.output(bot.id, "def");
    fake.emit({
      name: "terminal.data",
      params: { botId: bot.id, generation: 1, offset: 6, data: encodeText("ghi") },
    });
    fake.terminals.output(bot.id, "ghi");
    await settle();
    expect(fake.terminals.attaches.at(-1)).toEqual({ botId: bot.id, generation: 1, offset: 3 });
    expect(screen.text).toBe("abcdefghi");
    expect(screen.resets).toBe(1);
  });

  test("duplicated and old output is ignored", async () => {
    const { fake, bot, screen, session } = setup();
    fake.terminals.output(bot.id, "hello");
    session.start();
    await settle();
    const chunk = (generation: number, offset: number, text: string) =>
      fake.emit({
        name: "terminal.data",
        params: { botId: bot.id, generation, offset, data: encodeText(text) },
      });
    chunk(1, 0, "hel");
    chunk(1, 3, "lo!");
    chunk(0, 0, "stale generation");
    expect(screen.text).toBe("hello!");
    expect(screen.history).toBe("hello");
  });

  test("reconnecting resumes from the last byte instead of redrawing", async () => {
    const { fake, bot, screen, session } = setup();
    fake.terminals.output(bot.id, "one ");
    session.start();
    await settle();
    fake.setConnection({ kind: "waiting", retryAt: 0 });
    fake.terminals.output(bot.id, "two ");
    fake.setConnection({ kind: "open", daemonVersion: "0.1.0" });
    await settle();
    expect(fake.terminals.attaches.at(-1)).toEqual({ botId: bot.id, generation: 1, offset: 4 });
    expect(screen.text).toBe("one two ");
    expect(screen.resets).toBe(1);
  });

  test("input and size go to the daemon", async () => {
    const { fake, bot, session } = setup();
    session.start();
    await settle();
    session.input("olá\r");
    session.resize(100, 30);
    session.resize(100, 30);
    expect(fake.terminals.input).toEqual([{ botId: bot.id, text: "olá\r" }]);
    expect(fake.terminals.sizes.get(bot.id)).toEqual({ cols: 100, rows: 30 });
    expect(fake.calls.filter((call) => call.method === "terminal.resize")).toHaveLength(1);
    // Keys typed while disconnected are dropped, not sent late.
    fake.setConnection({ kind: "waiting", retryAt: 0 });
    session.input("x");
    expect(fake.terminals.input).toHaveLength(1);
  });
});
