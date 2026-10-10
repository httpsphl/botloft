import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "../../App";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
import { planPercent } from "../../lib/format";
import { periodStart } from "./TokensByBot";

afterEach(cleanup);

async function openUsage(fake: FakeBotloft) {
  render(<App host={new FakeHost()} connect={() => fake as Client} />);
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Usage" }));
  return screen.getByRole("dialog", { name: "Usage" });
}

describe("tokens by agent", () => {
  test("each agent's tokens add up over the period chosen, with the total", async () => {
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
    const list = await within(dialog).findByRole("list", { name: "Use by agent" });
    const rows = within(list).getAllByRole("listitem");
    // Each turn: 12 new, 1,800 into the cache and 640 written count; 24,000 reread do not.
    expect(rows.map((row) => row.textContent)).toEqual([
      "Scout4.9kOps · Worked 2 times",
      "Writer2.5kOps · Worked once",
      "All agents7.4kWorked 3 times",
    ]);

    fireEvent.click(within(dialog).getByRole("radio", { name: "7 days" }));
    expect(await within(dialog).findByText("4.9k")).toBeDefined();
    expect(dialog.textContent).toContain("Writer4.9kOps · Worked 2 times");
    expect(dialog.textContent).toContain("All agents9.8k");
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
    const list = await within(dialog).findByRole("list", { name: "Use by agent" });
    expect(
      within(list)
        .getAllByRole("listitem")
        .map((row) => row.textContent),
    ).toEqual(["Scout76.1kOps · Worked 2 times"]);
  });

  test("agents with the same name in two crews show their crew", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const news = fake.addCrew("Newsroom");
    fake.chat.turn(fake.addBot(ops.id, "Scout", "Finds sources").id);
    fake.chat.turn(fake.addBot(news.id, "Scout", "Finds stories").id);

    const dialog = await openUsage(fake);
    const list = await within(dialog).findByRole("list", { name: "Use by agent" });
    expect(
      within(list)
        .getAllByRole("listitem")
        .map((row) => row.textContent),
    ).toEqual([
      "Scout2.5kNewsroom · Worked once",
      "Scout2.5kOps · Worked once",
      "All agents4.9kWorked 2 times",
    ]);
  });

  test("a period with no work says so", async () => {
    const dialog = await openUsage(new FakeBotloft());
    expect(await within(dialog).findByText("No agent worked in this period.")).toBeDefined();
  });

  test("today starts at midnight; the others count back from now", () => {
    const now = new Date(2026, 8, 30, 15, 20).getTime();
    expect(periodStart("today", now)).toBe(new Date(2026, 8, 30).getTime());
    expect(periodStart("week", now)).toBe(now - 7 * 86_400_000);
    expect(periodStart("all", now)).toBe(0);
  });
});

describe("each agent's share of the weekly plan", () => {
  test("shows once learned, with the tokens beside it, and says it is an estimate", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.chat.turn(scout.id);
    fake.chat.turn(scout.id);
    // 4,904 tokens at 0.000001 each: 0.49% of the week.
    fake.planSharePerToken = 0.000_001;

    const dialog = await openUsage(fake);
    const list = await within(dialog).findByRole("list", { name: "Use by agent" });
    expect(within(list).getByRole("listitem").textContent).toBe(
      "Scout≈ 0.5% of the weekOps · Worked 2 times · 4.9k tokens",
    );
    expect(dialog.textContent).toContain("≈ Estimated:");
    fireEvent.click(within(dialog).getByRole("radio", { name: "Last hour" }));
    expect(await within(dialog).findByText("≈ 0.5% of the week")).toBeDefined();
  });

  test("until learned, the tokens and a note that the share comes later", async () => {
    const fake = new FakeBotloft();
    fake.chat.turn(fake.addBot(fake.addCrew("Ops").id, "Scout", "Finds sources").id);
    const dialog = await openUsage(fake);
    expect(await within(dialog).findByText(/shows up once the plan has gone up/)).toBeDefined();
  });

  test("a share reads as a short percentage", () => {
    expect(planPercent(0.045)).toBe("4.5%");
    expect(planPercent(0.123)).toBe("12%");
    expect(planPercent(0.0000001)).toBe("<0.1%");
    expect(planPercent(0)).toBe("0%");
  });
});
