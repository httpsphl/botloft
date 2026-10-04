import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { BotId, Schedule } from "../../lib/protocol.gen";
import { crewOpened, renderApp } from "../../test/app";

afterEach(cleanup);

function add(fake: FakeBotloft, botId: BotId, name: string, schedule: Schedule) {
  return fake.routines.handlers()["routines.create"]({
    botId,
    name,
    prompt: "Do it",
    schedule,
    timezone: "UTC",
  });
}

const weekdays: Schedule = { kind: "weekly", days: [1, 2, 3, 4, 5], time: "09:00" };

/** Two crews, three bots, routines on two of them. */
function setup() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const site = fake.addCrew("Site");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.addBot(ops.id, "Writer", "Writes");
  const watcher = fake.addBot(site.id, "Watcher", "Watches");
  add(fake, scout.id, "Morning summary", weekdays);
  const alerts = add(fake, watcher.id, "Camper van alerts", { kind: "interval", minutes: 30 });
  // Every 30 minutes: due before the morning summary.
  alerts.nextRunAt = fake.now + 30 * 60_000;
  add(fake, watcher.id, "On deploy", { kind: "signal", name: "deployed" });
  return fake;
}

async function openPage(fake: FakeBotloft) {
  renderApp(fake);
  await crewOpened("Ops");
  fireEvent.click(screen.getByRole("button", { name: "Routines" }));
  return screen.findByRole("heading", { level: 1, name: "Routines" });
}

describe("the routines page", () => {
  test("is in the sidebar only once there is a routine", async () => {
    const fake = new FakeBotloft();
    fake.addBot(fake.addCrew("Ops").id, "Scout", "Finds sources");
    renderApp(fake);
    await crewOpened("Ops");
    expect(screen.queryByRole("button", { name: "Routines" })).toBeNull();
  });

  test("lists what comes up across crews, interval routines as keeping watch", async () => {
    await openPage(setup());
    const coming = screen.getByRole("list", { name: "Coming up" });
    const rows = within(coming).getAllByRole("listitem");
    // A signal waits for one: it never comes up on its own.
    expect(rows.map((row) => row.querySelector(".font-medium")?.textContent)).toEqual([
      "Camper van alerts",
      "Morning summary",
    ]);
    expect(rows[0]?.textContent).toContain("Keeping watch");
    expect(rows[0]?.textContent).toContain("Every 30 minutes");
    expect(rows[1]?.textContent).toContain("Weekdays at 09:00");
    expect(rows[1]?.textContent).toContain("Scout");

    // A click opens the routine to edit.
    fireEvent.click(within(rows[1] as HTMLElement).getByRole("button"));
    expect(screen.getByRole("dialog", { name: "Edit Morning summary" })).toBeDefined();
  });

  test("groups each bot's routines under its mascot, folded on demand", async () => {
    await openPage(setup());
    const watcher = screen.getByRole("region", { name: "Watcher" });
    const list = within(watcher).getByRole("list", { name: "Watcher's routines" });
    expect(within(list).getAllByRole("listitem")).toHaveLength(2);
    // Writer has none, so no group.
    expect(screen.queryByRole("region", { name: "Writer" })).toBeNull();

    fireEvent.click(within(watcher).getByRole("button", { name: /Watcher/, expanded: true }));
    expect(within(watcher).queryByRole("list")).toBeNull();
    fireEvent.click(within(watcher).getByRole("button", { name: "New routine for Watcher" }));
    expect(screen.getByRole("dialog", { name: "New routine for Watcher" })).toBeDefined();
  });
});
