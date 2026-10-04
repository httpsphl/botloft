import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { BotId, Message, MessageId } from "../../lib/protocol.gen";
import glanceFrames from "../../mascot-glance.css?raw";
import wakeFrames from "../../mascot-wake.css?raw";
import { crewOpened, renderApp } from "../../test/app";
import { APP_OPENED } from "../../ui/motion";
import { BotAvatar } from "./BotAvatar";
import { ListAvatar } from "./ListAvatar";
import { ARRIVE_MS, MOMENT_MS } from "./mascotMoments";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.restoreAllMocks();
});

const svgOf = (container: HTMLElement) => container.querySelector("svg") as SVGSVGElement;

describe("moments", () => {
  test("wakes when its bot comes back, not when it shows up awake", () => {
    vi.useFakeTimers();
    const { container, rerender } = render(<BotAvatar color="#5ec8ff" mood="idle" />);
    expect(svgOf(container).hasAttribute("data-wake")).toBe(false);

    rerender(<BotAvatar color="#5ec8ff" mood="sleeping" />);
    expect(svgOf(container).hasAttribute("data-wake")).toBe(false);
    rerender(<BotAvatar color="#5ec8ff" mood="working" />);
    expect(svgOf(container).hasAttribute("data-wake")).toBe(true);
    act(() => vi.advanceTimersByTime(MOMENT_MS.wake));
    expect(svgOf(container).hasAttribute("data-wake")).toBe(false);
  });

  test("does not cheer when it only finished starting up", () => {
    const { container, rerender } = render(<BotAvatar color="#5ec8ff" mood="working" starting />);
    rerender(<BotAvatar color="#5ec8ff" mood="idle" />);
    expect(svgOf(container).hasAttribute("data-cheer")).toBe(false);

    // Started, then worked: finishing that work is worth a cheer.
    rerender(<BotAvatar color="#5ec8ff" mood="working" starting />);
    rerender(<BotAvatar color="#5ec8ff" mood="working" />);
    rerender(<BotAvatar color="#5ec8ff" mood="idle" />);
    expect(svgOf(container).hasAttribute("data-cheer")).toBe(true);
  });

  test("a new bot that starts up while it pops in does not wake on top of it", () => {
    vi.useFakeTimers();
    vi.setSystemTime(APP_OPENED + 60_000);
    const created = { id: "bot_new" as BotId, color: "#9be564", paused: false };
    const createdAt = APP_OPENED + 59_000;
    const { container, rerender } = render(
      <ListAvatar bot={{ ...created, state: "offline", createdAt }} crewPaused={false} size={32} />,
    );
    rerender(
      <ListAvatar
        bot={{ ...created, state: "launching", createdAt }}
        crewPaused={false}
        size={32}
      />,
    );
    expect(svgOf(container).hasAttribute("data-arrive")).toBe(true);
    expect(svgOf(container).hasAttribute("data-wake")).toBe(false);
  });

  test("a bot created a moment ago pops in once; an older one does not", () => {
    vi.useFakeTimers();
    vi.setSystemTime(APP_OPENED + 60_000);
    const bot = (createdAt: number) => ({
      id: "bot_new" as BotId,
      color: "#9be564",
      state: "launching" as const,
      paused: false,
      createdAt,
    });
    const fresh = render(
      <ListAvatar bot={bot(APP_OPENED + 59_000)} crewPaused={false} size={32} />,
    );
    expect(svgOf(fresh.container).hasAttribute("data-arrive")).toBe(true);
    act(() => vi.advanceTimersByTime(ARRIVE_MS));
    expect(svgOf(fresh.container).hasAttribute("data-arrive")).toBe(false);

    const old = render(<ListAvatar bot={bot(APP_OPENED - 1)} crewPaused={false} size={32} />);
    expect(svgOf(old.container).hasAttribute("data-arrive")).toBe(false);
  });

  test("every moment ends at rest, so less motion shows none of it", () => {
    const end = (frames: string, name: string) =>
      frames
        .match(new RegExp(`@keyframes ${name} \\{([\\s\\S]*?)\\n\\}`))?.[1]
        ?.split(/100% \{/)
        .at(-1);
    expect(end(wakeFrames, "wake-up")).toContain("transform: scale(1, 1);");
    expect(end(wakeFrames, "wake-open")).toMatch(/opacity: 1;\s*clip-path: inset\(0\);/);
    expect(end(wakeFrames, "wake-shut")).toContain("opacity: 0;");
    expect(end(wakeFrames, "arrive-pop")).toMatch(/opacity: 1;\s*transform: scale\(1, 1\);/);
    expect(end(wakeFrames, "arrive-open")).toContain("clip-path: inset(0);");
    expect(end(glanceFrames, "glance")).toContain("transform: translate(0, 0);");
  });
});

describe("glances", () => {
  test("two bots look at each other when one writes to the other", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    const writer = fake.addBot(crew.id, "Writer");
    // Scout sits above Writer on screen.
    const rows: Record<string, number> = { [scout.id]: 100, [writer.id]: 200 };
    vi.spyOn(Element.prototype, "getBoundingClientRect").mockImplementation(function (
      this: Element,
    ) {
      const top = rows[this.getAttribute("data-bot") ?? ""];
      return top === undefined ? new DOMRect(0, 0, 0, 0) : new DOMRect(20, top, 32, 32);
    });
    renderApp(fake);
    await crewOpened("Ops");
    const mascots = (botId: BotId) => [...document.querySelectorAll(`svg[data-bot="${botId}"]`)];

    const message: Message = {
      id: "msg_glance" as MessageId,
      crewId: crew.id,
      fromKind: "bot",
      fromBotId: scout.id,
      toBotId: writer.id,
      kind: "note",
      body: "",
      taskId: null,
      routineId: null,
      questionId: null,
      attachments: [],
      createdAt: Date.now(),
    };
    act(() => fake.emit({ name: "message.created", params: message }));

    const looking = (botId: BotId) => mascots(botId).find((svg) => svg.hasAttribute("data-glance"));
    await waitFor(() => expect(looking(writer.id)).toBeDefined());
    // Writer looks up at Scout; Scout looks down at Writer.
    expect((looking(writer.id) as SVGElement).style.getPropertyValue("--glance-y")).toBe("-22px");
    expect((looking(scout.id) as SVGElement).style.getPropertyValue("--glance-y")).toBe("22px");
    await waitFor(() => expect(looking(writer.id)).toBeUndefined(), { timeout: 3000 });
  });
});
