import { act, cleanup, fireEvent, screen } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

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
