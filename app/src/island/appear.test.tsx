import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { prefs, resetPrefs } from "../shell/prefs";
import { FakeIslandHost } from "./fakeHost";
import { IslandApp } from "./IslandApp";
import { LEAVE_MS, PEEK_MS } from "./useAppear";

afterEach(() => {
  cleanup();
  resetPrefs();
});

const LABEL = "Botloft at the top of the screen";
const wait = (ms: number) => act(() => new Promise((resolve) => setTimeout(resolve, ms)));

/** A crew where Writer works, so the pill shows; the island already read it. */
async function working(host = new FakeIslandHost()) {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  fake.setBotState(writer.id, "busy");
  render(<IslandApp api={fake} host={host} />);
  await screen.findByRole("button", { name: "1 working" });
  return { fake, host, scout, writer, ops };
}

const ask = (fake: FakeBotloft, botId: string) =>
  act(() => {
    fake.chat.ask(botId, "Bash", "git status", JSON.stringify({ command: "git status" }));
    fake.setBotState(botId, "needs_approval");
  });

describe("when the island comes down", () => {
  test("opens on Home when a new request arrives", async () => {
    const { fake, scout } = await working();
    ask(fake, scout.id);
    const island = await screen.findByRole("region", { name: LABEL });
    expect(await within(island).findByText("Run a command")).toBeDefined();
    expect(within(island).getByRole("tab", { name: "Home" }).getAttribute("aria-selected")).toBe(
      "true",
    );
  });

  test("what waited when it started is no news: it stays a pill", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.chat.ask(scout.id, "Bash", "git status", JSON.stringify({ command: "git status" }));
    render(<IslandApp api={fake} host={new FakeIslandHost()} />);
    await screen.findByRole("button", { name: "1 waiting" });
    await wait(300);
    expect(screen.queryByText("Run a command")).toBeNull();
  });

  test("stays a pill while Botloft is in front, or Windows keeps the screen quiet", async () => {
    const host = new FakeIslandHost();
    host.front = true;
    const { fake, scout, writer } = await working(host);
    ask(fake, scout.id);
    await screen.findByRole("button", { name: "1 waiting, 1 working" });
    await wait(300);
    expect(screen.queryByText("Run a command")).toBeNull();

    host.front = false;
    host.quietNow = true;
    ask(fake, writer.id);
    await screen.findByRole("button", { name: "2 waiting" });
    await wait(300);
    expect(screen.queryByText("Run a command")).toBeNull();
  });

  test("peeks while the mouse rests on the pill, and a click keeps it open", async () => {
    await working();
    const island = screen.getByRole("region", { name: LABEL });
    fireEvent.mouseEnter(island);
    await wait(PEEK_MS + 100);
    expect(within(island).getByRole("tablist")).toBeDefined();
    fireEvent.mouseLeave(island);
    await wait(LEAVE_MS + 150);
    expect(within(island).queryByRole("tablist")).toBeNull();

    fireEvent.mouseEnter(island);
    await wait(PEEK_MS + 100);
    fireEvent.pointerDown(island);
    fireEvent.mouseLeave(island);
    await wait(LEAVE_MS + 150);
    expect(within(island).getByRole("tablist")).toBeDefined();
  });

  test("hides when turned off, and sits where the owner chose", async () => {
    const { host } = await working();
    await waitFor(() => expect(host.calls).toContain("show 200x36 primary"));
    act(() => prefs.islandScreen.set("mouse"));
    await waitFor(() => expect(host.calls).toContain("show 200x36 mouse"));
    act(() => prefs.island.set(false));
    await waitFor(() => expect(screen.queryByRole("region")).toBeNull());
    expect(host.calls.at(-1)).toBe("hide");
  });

  test("no cheer on a quiet screen", async () => {
    const host = new FakeIslandHost();
    host.quietNow = true;
    const { fake, writer } = await working(host);
    act(() => fake.setBotState(writer.id, "idle"));
    await wait(200);
    expect(screen.queryByText("Writer finished")).toBeNull();
  });

  test("pauses and resumes every crew from the panel", async () => {
    const { fake, ops } = await working();
    fireEvent.click(screen.getByRole("button", { name: "1 working" }));
    fireEvent.click(await screen.findByRole("button", { name: "Pause every crew" }));
    await waitFor(() => expect(fake.crew(ops.id).paused).toBe(true));
    fireEvent.click(await screen.findByRole("button", { name: "Resume every crew" }));
    await waitFor(() => expect(fake.crew(ops.id).paused).toBe(false));
  });
});
