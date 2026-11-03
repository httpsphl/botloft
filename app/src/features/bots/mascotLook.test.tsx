import { act, cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { Activity, BotId, ChatItem } from "../../lib/protocol.gen";
import { ApiProvider } from "../../store/context";
import { BotAvatar } from "./BotAvatar";
import { doingNow, doingOf, faceNow, useMascotLook, WORRIED_MS } from "./mascotLook";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

const tool = (name: string): Activity =>
  ({ kind: "tool", text: "", tool: name, at: 0 }) as unknown as Activity;

describe("what a mascot shows", () => {
  test("the tool in use picks the drawing", () => {
    expect(doingOf("WebSearch")).toBe("search");
    expect(doingOf("mcp__botloft__browser_open")).toBe("browse");
    expect(doingOf("Edit")).toBe("write");
    expect(doingOf("Bash")).toBe("run");
    expect(doingOf("Grep")).toBe("read");
    expect(doingOf("mcp__botloft__desktop_click")).toBe("control");
    expect(doingOf("mcp__notion__search_pages")).toBe("search");
    expect(doingOf("mcp__botloft__send_message")).toBe("think");
  });

  test("working it shows its tool or thinks; waiting, a '!'; otherwise nothing", () => {
    expect(doingNow("working", tool("Write"))).toBe("write");
    expect(doingNow("working", null)).toBe("think");
    expect(doingNow("waiting", tool("Write"))).toBe("ask");
    expect(doingNow("idle", tool("Write"))).toBeNull();
    expect(doingNow("awake", null)).toBeNull();
  });

  test("its face: worried after a failure, sleepy after a long wait, focused at work", () => {
    const calm = { worried: false, longWait: false };
    expect(faceNow("working", "run", calm)).toBe("focus");
    expect(faceNow("working", "think", calm)).toBeNull();
    expect(faceNow("working", "run", { ...calm, worried: true })).toBe("worried");
    expect(faceNow("sleeping", null, { ...calm, worried: true })).toBeNull();
    expect(faceNow("waiting", "ask", { ...calm, longWait: true })).toBe("sleepy");
  });

  test("a badge shows only from 24 px, and its face on the mascot", () => {
    const { container, rerender } = render(
      <BotAvatar color="#5ec8ff" size={32} mood="working" doing="write" face="focus" />,
    );
    expect(container.querySelector(".avatar-badge")?.getAttribute("data-doing")).toBe("write");
    expect(container.querySelector("svg")?.dataset.face).toBe("focus");
    rerender(<BotAvatar color="#5ec8ff" size={20} mood="working" doing="write" />);
    expect(container.querySelector(".avatar-badge")).toBeNull();
  });

  test("a failed tool makes its bot's mascot worried for a moment", () => {
    vi.useFakeTimers();
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const bot = fake.addBot(crew.id, "Scout");
    let face: string | null = null;
    function Probe({ botId }: { botId: BotId }) {
      face = useMascotLook(botId, "working").face;
      return null;
    }
    render(
      <ApiProvider api={fake}>
        <Probe botId={bot.id} />
      </ApiProvider>,
    );
    const failed = {
      id: "cht_1",
      botId: bot.id,
      body: { kind: "tool", status: "failed", name: "Bash" },
    } as unknown as ChatItem;
    act(() => fake.emit({ name: "chat.item", params: { item: failed, activity: null } }));
    expect(face).toBe("worried");
    act(() => vi.advanceTimersByTime(WORRIED_MS));
    expect(face).toBeNull();
  });
});
