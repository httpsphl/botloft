import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "../../App";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
import { dollars, planPercent } from "../../lib/format";
import type { BotId, RoutineId } from "../../lib/protocol.gen";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { periodStart } from "./AgentCosts";

afterEach(cleanup);

async function openUsage(fake: FakeBotloft) {
  render(<App host={new FakeHost()} connect={() => fake as Client} />);
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Usage" }));
  return screen.getByRole("dialog", { name: "Usage" });
}

async function rowsOf(dialog: HTMLElement) {
  const list = await within(dialog).findByRole("list", { name: "What the agents cost" });
  return within(list)
    .getAllByRole("listitem")
    .map((row) => row.textContent);
}

/** A model's part of a turn's cost, at the fake's clock. */
function cost(
  fake: FakeBotloft,
  botId: BotId,
  amount: number,
  model: string | null = null,
  routineId: RoutineId | null = null,
) {
  fake.costParts.push({ botId, at: fake.now, cost: amount, model, routineId });
}

/** Scout and Writer on Claude, Cody on Codex with no price. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  const cody = fake.addBot(ops.id, "Cody", "Codes");
  fake.bot(cody.id).agent = "codex";
  const now = fake.now;
  fake.now = now - 3 * 86_400_000;
  fake.chat.turn(writer.id);
  cost(fake, writer.id, 0.5, "claude-opus-5-5");
  fake.now = now;
  fake.chat.turn(scout.id);
  fake.chat.turn(scout.id);
  cost(fake, scout.id, 0.25, "claude-opus-5-5");
  cost(fake, scout.id, 0.05, "claude-haiku-5-5");
  fake.chat.turn(writer.id);
  cost(fake, writer.id, 0.1);
  fake.chat.turn(cody.id);
  return { fake, scout, writer };
}

describe("what the agents cost", () => {
  test("each agent's API price, its part of the total, and the agents with no price", async () => {
    const { fake } = crew();
    const dialog = await openUsage(fake);
    expect(await rowsOf(dialog)).toEqual([
      "Scout$0.30Ops · Worked 2 times · 75% of the total",
      "Writer$0.10Ops · Worked once · 25% of the total",
      "Cody2.5k tokens · no priceOps · Worked once",
      "Total$0.40",
    ]);
    expect(dialog.textContent).toContain("Your plan does not charge this amount");

    fireEvent.click(within(dialog).getByRole("radio", { name: "7 days" }));
    expect(await within(dialog).findByText("$0.90")).toBeDefined();
    expect((await rowsOf(dialog))[0]).toBe("Writer$0.60Ops · Worked 2 times · 67% of the total");
  });

  test("by model, by routine and by day", async () => {
    const { fake, scout } = crew();
    const routine = await fake.call("routines.create", {
      botId: scout.id,
      name: "Morning news",
      prompt: "Summarize",
      schedule: { kind: "manual" },
      timezone: "UTC",
    });
    cost(fake, scout.id, 0.2, "claude-sonnet-5-5", routine.id);
    const dialog = await openUsage(fake);
    await rowsOf(dialog);

    fireEvent.click(within(dialog).getByRole("radio", { name: "Models" }));
    expect(await within(dialog).findByText("Opus 5.5")).toBeDefined();
    expect(await rowsOf(dialog)).toEqual([
      "Opus 5.5$0.2542% of the total",
      "Sonnet 5.5$0.2033% of the total",
      "Model not recorded (older work)$0.1017% of the total",
      "Haiku 5.5$0.058.3% of the total",
      "Total$0.60",
    ]);

    fireEvent.click(within(dialog).getByRole("radio", { name: "Routines" }));
    expect(await within(dialog).findByText("Morning news")).toBeDefined();
    expect(await rowsOf(dialog)).toEqual([
      "Chats and requests$0.4067% of the total",
      "Morning news$0.20Scout · 33% of the total",
      "Total$0.60",
    ]);

    fireEvent.click(within(dialog).getByRole("radio", { name: "Days" }));
    fireEvent.click(within(dialog).getByRole("radio", { name: "7 days" }));
    expect(await within(dialog).findByText("$1.10")).toBeDefined();
    const days = await rowsOf(dialog);
    expect(days).toHaveLength(3);
    expect(days[0]).toContain("$0.60");
    expect(days[1]).toContain("$0.50");
  });

  test("agents with the same name in two crews show their crew", async () => {
    const fake = new FakeBotloft();
    const scoutA = fake.addBot(fake.addCrew("Ops").id, "Scout", "Finds sources");
    const scoutB = fake.addBot(fake.addCrew("Newsroom").id, "Scout", "Finds stories");
    fake.chat.turn(scoutA.id);
    fake.chat.turn(scoutB.id);
    cost(fake, scoutA.id, 0.1);
    cost(fake, scoutB.id, 0.3);
    const dialog = await openUsage(fake);
    expect(await rowsOf(dialog)).toEqual([
      "Scout$0.30Newsroom · Worked once · 75% of the total",
      "Scout$0.10Ops · Worked once · 25% of the total",
      "Total$0.40",
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

  test("a cost reads in dollars, and a part as a short percentage", () => {
    expect(dollars(12.4)).toBe("$12.40");
    expect(dollars(0.004)).toBe("< $0.01");
    expect(dollars(0)).toBe("$0.00");
    expect(planPercent(0.045)).toBe("4.5%");
    expect(planPercent(0.0000001)).toBe("<0.1%");
  });
});

describe("the turn line", () => {
  test("says the turn's API price, with the tokens in its title", async () => {
    const fake = new FakeBotloft();
    const scout = fake.addBot(fake.addCrew("Ops").id, "Scout", "Finds sources");
    fake.chat.turn(scout.id, null, undefined, 42_000);
    fake.chat.turn(scout.id, null, undefined, 3_000);
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const line = await screen.findByText("Done in 4.2 s · $0.04");
    expect(line.title).toBe(
      "Took 4.2 s. Read 1.8k tokens and wrote 640. At API prices: $0.04, included in your plan.",
    );
    expect(screen.getByText("Done in 4.2 s · < $0.01")).toBeDefined();
  });
});
