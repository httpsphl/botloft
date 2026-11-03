import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { prefs, resetPrefs } from "../shell/prefs";
import { FakeIslandHost } from "./fakeHost";
import { IslandApp } from "./IslandApp";

afterEach(() => {
  cleanup();
  resetPrefs();
});

const LABEL = "Botloft at the top of the screen";

/** "Ops" with Scout, nothing going on, and a shortcut; the island is up. */
async function quiet(host = new FakeIslandHost()) {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  fake.routines.handlers()["routines.create"]({
    botId: scout.id,
    name: "Morning summary",
    prompt: "Summarize what arrived",
    schedule: { kind: "manual" },
    timezone: "UTC",
  });
  render(<IslandApp api={fake} host={host} />);
  await waitFor(() => expect(host.calls).toContain("summon 1000"));
  return { fake, host, scout };
}

describe("calling the island with the right Ctrl (spec 31.6)", () => {
  test("opens it on Home with nothing going on, and calling again hides it", async () => {
    const { host } = await quiet();
    expect(screen.queryByRole("region", { name: LABEL })).toBeNull();
    act(() => host.summon());
    const island = await screen.findByRole("region", { name: LABEL });
    expect(within(island).getByRole("tab", { name: "Home" }).getAttribute("aria-selected")).toBe(
      "true",
    );
    act(() => host.summon());
    await waitFor(() => expect(screen.queryByRole("region", { name: LABEL })).toBeNull());
  });

  test("follows the settings: off, or held longer", async () => {
    const { host } = await quiet();
    act(() => prefs.islandHold.set("2000"));
    await waitFor(() => expect(host.calls).toContain("summon 2000"));
    act(() => prefs.islandSummon.set(false));
    await waitFor(() => expect(host.calls).toContain("summon off"));
    act(() => prefs.islandSummon.set(true));
    act(() => prefs.island.set(false));
    await waitFor(() => expect(host.calls.at(-1)).toBe("summon off"));
  });

  test("with Botloft in front it steps aside, unless the owner called it", async () => {
    const { fake, host, scout } = await quiet();
    act(() => fake.setBotState(scout.id, "busy"));
    await screen.findByRole("button", { name: "1 working" });
    act(() => host.setFront(true));
    await waitFor(() => expect(screen.queryByRole("button", { name: "1 working" })).toBeNull());
    act(() => host.summon());
    expect(await screen.findByRole("region", { name: LABEL })).toBeDefined();
    act(() => host.summon());
    await waitFor(() => expect(screen.queryByRole("region", { name: LABEL })).toBeNull());
    act(() => host.setFront(false));
    expect(await screen.findByRole("button", { name: "1 working" })).toBeDefined();

    // Turned off in Settings, it stays with Botloft in front.
    act(() => prefs.islandHideWithApp.set(false));
    act(() => host.setFront(true));
    expect(await screen.findByRole("button", { name: "1 working" })).toBeDefined();
  });
});

describe("Home on the island (spec 31.4)", () => {
  test("asks an agent for something in one line, then follows the chat", async () => {
    const { fake, host, scout } = await quiet();
    act(() => host.summon());
    const island = await screen.findByRole("region", { name: LABEL });
    fireEvent.change(within(island).getByRole("textbox", { name: "Ask Scout for something" }), {
      target: { value: "Find three sources on solar panels" },
    });
    fireEvent.click(within(island).getByRole("button", { name: "Send" }));
    expect((await within(island).findByRole("status")).textContent).toContain("Sent to Scout");
    expect(fake.calls.find((call) => call.method === "messages.send")?.params).toMatchObject({
      botId: scout.id,
      body: "Find three sources on solar panels",
    });
    fireEvent.click(within(island).getByRole("button", { name: "See the chat" }));
    expect(within(island).getByRole("tab", { name: "Chat" }).getAttribute("aria-selected")).toBe(
      "true",
    );
  });

  test("runs a shortcut with one click", async () => {
    const { fake, host } = await quiet();
    act(() => host.summon());
    const island = await screen.findByRole("region", { name: LABEL });
    const shortcuts = await within(island).findByRole("region", { name: "Shortcuts" });
    fireEvent.click(
      within(shortcuts).getByRole("button", { name: "Run Morning summary, by Scout" }),
    );
    expect(await within(shortcuts).findByText("Started")).toBeDefined();
    expect(fake.calls.some((call) => call.method === "routines.runNow")).toBe(true);
  });
});
