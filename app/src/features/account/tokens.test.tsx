import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "../../App";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
import { periodStart } from "./TokensByBot";

afterEach(cleanup);

async function openUsage(fake: FakeBotloft) {
  render(<App host={new FakeHost()} connect={() => fake as Client} />);
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Usage" }));
  return screen.getByRole("dialog", { name: "Usage" });
}

describe("tokens by bot", () => {
  test("each bot's tokens add up over the period chosen, with the total", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    const writer = fake.addBot(ops.id, "Writer", "Writes");
    const now = fake.now;
    fake.now = now - 3 * 86_400_000;
    fake.chat.turn(writer.id);
    fake.now = now;
    fake.chat.turn(scout.id);
    fake.chat.turn(scout.id);
    fake.chat.turn(writer.id);

    const dialog = await openUsage(fake);
    const list = await within(dialog).findByRole("list", { name: "Tokens by bot" });
    const rows = within(list).getAllByRole("listitem");
    // Each turn: 12 new, 1,800 into the cache and 640 written count; 24,000 reread do not.
    expect(rows.map((row) => row.textContent)).toEqual([
      "Scout4.9kOps · Worked 2 times",
      "Writer2.5kOps · Worked once",
      "All bots7.4kWorked 3 times",
    ]);

    fireEvent.click(within(dialog).getByRole("radio", { name: "7 days" }));
    expect(await within(dialog).findByText("4.9k")).toBeDefined();
    expect(dialog.textContent).toContain("Writer4.9kOps · Worked 2 times");
    expect(dialog.textContent).toContain("All bots9.8k");
  });

  test("the conversation sent again after a pause counts in the total", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.chat.turn(scout.id, null, {
      input: 2,
      cacheWrite: 72_636,
      reloaded: 72_000,
      cacheRead: 0,
      output: 983,
    });
    fake.chat.turn(scout.id);

    const dialog = await openUsage(fake);
    const list = await within(dialog).findByRole("list", { name: "Tokens by bot" });
    expect(
      within(list)
        .getAllByRole("listitem")
        .map((row) => row.textContent),
    ).toEqual(["Scout76.1kOps · Worked 2 times"]);
  });

  test("bots with the same name in two crews show their crew", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const news = fake.addCrew("Newsroom");
    fake.chat.turn(fake.addBot(ops.id, "Scout", "Finds sources").id);
    fake.chat.turn(fake.addBot(news.id, "Scout", "Finds stories").id);

    const dialog = await openUsage(fake);
    const list = await within(dialog).findByRole("list", { name: "Tokens by bot" });
    expect(
      within(list)
        .getAllByRole("listitem")
        .map((row) => row.textContent),
    ).toEqual([
      "Scout2.5kNewsroom · Worked once",
      "Scout2.5kOps · Worked once",
      "All bots4.9kWorked 2 times",
    ]);
  });

  test("a period with no work says so", async () => {
    const dialog = await openUsage(new FakeBotloft());
    expect(await within(dialog).findByText("No bot worked in this period.")).toBeDefined();
  });

  test("today starts at midnight; the others count back from now", () => {
    const now = new Date(2026, 8, 30, 15, 20).getTime();
    expect(periodStart("today", now)).toBe(new Date(2026, 8, 30).getTime());
    expect(periodStart("week", now)).toBe(now - 7 * 86_400_000);
    expect(periodStart("all", now)).toBe(0);
  });
});
