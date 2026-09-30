import { act, cleanup, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, beforeAll, describe, expect, test, vi } from "vitest";
import { useBotFiles } from "../features/files/useBotFiles";
import { FakeBotloft } from "../lib/fake";
import { createAppStore } from "../store/app";
import { DaemonProvider } from "../store/context";
import { syncStore } from "../store/sync";
import { markHidden, watchVisibility, windowHidden } from "./visibility";

beforeAll(watchVisibility);

afterEach(() => {
  markHidden(false);
  cleanup();
  vi.useRealTimers();
});

const polls = (fake: FakeBotloft, method: string) =>
  fake.calls.filter((call) => call.method === method).length;

describe("window visibility", () => {
  test("hiding the window pauses the animations until it comes back", () => {
    markHidden(true);
    expect(windowHidden()).toBe(true);
    expect(document.documentElement.hasAttribute("data-hidden")).toBe(true);
    window.dispatchEvent(new Event("focus"));
    expect(windowHidden()).toBe(false);
    expect(document.documentElement.hasAttribute("data-hidden")).toBe(false);
  });

  test("the status is not polled while the window is hidden, and is read when it comes back", async () => {
    vi.useFakeTimers();
    const fake = new FakeBotloft();
    const stop = syncStore(createAppStore(fake), fake);
    await vi.advanceTimersByTimeAsync(0);
    const before = polls(fake, "system.status");
    markHidden(true);
    await vi.advanceTimersByTimeAsync(60_000);
    expect(polls(fake, "system.status")).toBe(before);
    markHidden(false);
    await vi.advanceTimersByTimeAsync(0);
    expect(polls(fake, "system.status")).toBe(before + 1);
    stop();
  });

  test("a working bot's files are read over and over only while their panel is in sight", async () => {
    vi.useFakeTimers();
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    const wrapper = ({ children }: { children: ReactNode }) => (
      <DaemonProvider api={fake}>{children}</DaemonProvider>
    );
    const busy = { id: scout.id, state: "busy" as const };
    const { rerender } = renderHook(({ watching }) => useBotFiles(busy, watching), {
      wrapper,
      initialProps: { watching: false },
    });
    await act(() => vi.advanceTimersByTimeAsync(0));
    const opened = polls(fake, "files.list");
    await act(() => vi.advanceTimersByTimeAsync(20_000));
    expect(polls(fake, "files.list")).toBe(opened);

    rerender({ watching: true });
    await act(() => vi.advanceTimersByTimeAsync(8_000));
    expect(polls(fake, "files.list")).toBeGreaterThanOrEqual(opened + 3);

    act(() => markHidden(true));
    const hidden = polls(fake, "files.list");
    await act(() => vi.advanceTimersByTimeAsync(20_000));
    expect(polls(fake, "files.list")).toBe(hidden);
  });
});
