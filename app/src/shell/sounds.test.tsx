import { act, cleanup, fireEvent, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import type { BotFile } from "../lib/protocol.gen";
import { crewOpened, openBot, renderApp } from "../test/app";
import { prefs, resetPrefs } from "./prefs";
import { mayPlay, playSound, resetSounds } from "./sounds";
import { tones } from "./tones";

vi.mock("./tones", () => ({ tones: vi.fn() }));

const played = () => vi.mocked(tones).mock.calls.map(([sound]) => sound);

beforeEach(() => {
  vi.mocked(tones).mockClear();
  resetSounds();
});

afterEach(() => {
  cleanup();
  resetPrefs();
});

async function scoutOpen() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  fake.setBotState(scout.id, "idle");
  fake.setBotState(writer.id, "idle");
  const host = new FakeHost();
  renderApp(fake, host);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  // Later than the chat opening, as anything new from the daemon is.
  fake.now = Date.now() + 1000;
  return { fake, host, ops, scout, writer };
}

const report: BotFile = {
  path: "C:\\Work\\report.md",
  name: "report.md",
  mediaType: "text/markdown",
  size: 476,
  modifiedAt: 0,
} as BotFile;

describe("app sounds", () => {
  test("a message sent ticks, unless the owner turned app sounds off", async () => {
    await scoutOpen();
    const field = screen.getByLabelText("Message to Scout");
    fireEvent.change(field, { target: { value: "Hi" } });
    fireEvent.keyDown(field, { key: "Enter" });
    await screen.findByText("Hi");
    expect(played()).toEqual(["sent"]);
    prefs.appSounds.set(false);
    resetSounds();
    fireEvent.change(field, { target: { value: "Again" } });
    fireEvent.keyDown(field, { key: "Enter" });
    await screen.findByText("Again");
    expect(played()).toEqual(["sent"]);
  });

  test("a reply or a file in the open chat, but not in another one", async () => {
    const { fake, scout, writer } = await scoutOpen();
    act(() => {
      fake.chat.reply(writer.id, "Elsewhere");
    });
    expect(played()).toEqual([]);
    act(() => {
      fake.chat.reply(scout.id, "Here");
    });
    expect(played()).toEqual(["reply"]);
    resetSounds();
    act(() => {
      const call = fake.chat.tool(scout.id, "mcp__botloft__share_file");
      fake.chat.finish(call, JSON.stringify({ shown: [report] }));
    });
    expect(played()).toEqual(["reply", "file"]);
  });

  test("a spark for a new bot, and nothing while Botloft is behind", async () => {
    const { fake, host, ops } = await scoutOpen();
    act(() => {
      fake.addBot(ops.id, "Clerk", "Files things");
    });
    expect(played()).toEqual(["spark"]);
    resetSounds();
    host.front = false;
    act(() => {
      fake.addBot(ops.id, "Planner", "Plans");
    });
    expect(played()).toEqual(["spark"]);
  });

  test("never two within a moment, except a bot that needs the owner", () => {
    playSound("reply");
    playSound("file");
    playSound("needs");
    expect(played()).toEqual(["reply", "needs"]);
    expect(mayPlay("spark", Date.now() + 1500)).toBe(true);
    prefs.sound.set(false);
    expect(mayPlay("needs", Date.now())).toBe(false);
  });
});
