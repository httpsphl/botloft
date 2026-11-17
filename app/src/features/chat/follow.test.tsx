import { act, cleanup, fireEvent, screen } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { followTop } from "./follow";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("following the end of the chat", () => {
  test("keeps the end in view as the chat grows, and leaves who scrolled up", async () => {
    let grew = () => {};
    vi.stubGlobal(
      "ResizeObserver",
      class {
        constructor(callback: ResizeObserverCallback) {
          grew = () => callback([], this as unknown as ResizeObserver);
        }
        observe() {}
        disconnect() {}
      },
    );
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "idle");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const list = await screen.findByRole("list", { name: "Messages" });
    const scroller = list.closest(".overflow-y-auto") as HTMLElement;
    let height = 5000;
    Object.defineProperty(scroller, "scrollHeight", { get: () => height });

    // The dots of a bot that starts working make the chat taller.
    act(() => fake.setBotState(scout.id, "busy"));
    act(() => grew());
    expect(scroller.scrollTop).toBe(5000);

    // Reading up the chat: it stays where it is.
    scroller.scrollTop = 0;
    fireEvent.scroll(scroller);
    height = 6000;
    act(() => grew());
    expect(scroller.scrollTop).toBe(0);

    // Back at the end: it follows again.
    scroller.scrollTop = 5990;
    fireEvent.scroll(scroller);
    height = 7000;
    act(() => grew());
    expect(scroller.scrollTop).toBe(7000);
  });
});

describe("a reply taller than the chat", () => {
  /** A chat 600 px tall, scrolled to 1000, whose newest row is the bot's. */
  function chat(reply: { top: number; height: number }, newest = "bot") {
    const scroller = document.createElement("div");
    const list = document.createElement("ol");
    const row = document.createElement("li");
    const text = document.createElement("div");
    if (newest === "bot") {
      text.dataset.reply = "";
    }
    row.append(text);
    list.append(row);
    scroller.append(list);
    Object.defineProperty(scroller, "clientHeight", { value: 600 });
    Object.defineProperty(scroller, "scrollHeight", { value: 5000 });
    scroller.scrollTop = 1000;
    scroller.getBoundingClientRect = () => new DOMRect(0, 0, 800, 600);
    text.getBoundingClientRect = () => new DOMRect(0, reply.top, 600, reply.height);
    return followTop(scroller, list);
  }

  test("is followed from its start, with the bot's name above it", () => {
    expect(chat({ top: 200, height: 900 })).toBe(1000 + 200 - 80);
  });

  test("one that fits, one read past its start, or the owner's message: the end", () => {
    expect(chat({ top: 200, height: 300 })).toBe(5000);
    expect(chat({ top: -40, height: 900 })).toBe(5000);
    expect(chat({ top: 200, height: 900 }, "owner")).toBe(5000);
  });
});
