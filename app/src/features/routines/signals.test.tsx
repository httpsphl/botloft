import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { en } from "../../i18n/en";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, openTab, renderApp } from "../../test/app";
import { describeSchedule } from "./describe";

afterEach(cleanup);

/** A crew "Ops" with @scout and @writer, open on Scout's routines. */
async function openRoutines() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout");
  const writer = fake.addBot(ops.id, "Writer");
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  openTab(/^Routines/);
  return { fake, scout, writer };
}

describe("routines that wait for a signal", () => {
  test("the schedule reads as the signal", () => {
    expect(describeSchedule({ kind: "signal", name: "report-ready" }, en.routines.when)).toBe(
      "When a bot signals “report-ready”",
    );
  });

  test("the owner makes one, and a bot's signal runs it", async () => {
    const { fake, writer } = await openRoutines();
    fireEvent.click(screen.getByRole("button", { name: "New routine" }));
    const dialog = screen.getByRole("dialog", { name: "New routine for Scout" });
    fireEvent.change(within(dialog).getByLabelText("Name"), { target: { value: "Review" } });
    fireEvent.change(within(dialog).getByLabelText("What should Scout do?"), {
      target: { value: "Review the report" },
    });
    fireEvent.click(within(dialog).getByRole("radio", { name: "When a bot signals" }));
    expect(within(dialog).queryByLabelText("At")).toBeNull();
    expect(within(dialog).getByText(/Scout runs this when a bot of the crew/)).toBeDefined();
    fireEvent.change(within(dialog).getByLabelText("Signal"), {
      target: { value: "Report Ready" },
    });
    // No time: the zone and "computer off" have nothing to say.
    fireEvent.click(within(dialog).getByText("More options"));
    expect(within(dialog).queryByLabelText("Time zone")).toBeNull();
    expect(within(dialog).queryByLabelText("Use a cron expression")).toBeNull();
    fireEvent.click(within(dialog).getByRole("button", { name: "Create routine" }));

    await screen.findByText("Review");
    const params = fake.calls.find((call) => call.method === "routines.create")?.params;
    expect(params).toMatchObject({ schedule: { kind: "signal", name: "Report Ready" } });
    expect(screen.getByText(/When a bot signals “report-ready”/)).toBeDefined();
    expect(screen.getByText(/Waits for a signal/)).toBeDefined();

    act(() => {
      fake.routines.signal(writer.id, "report-ready", "In shared/report.md");
    });
    expect(await screen.findByText("Running now")).toBeDefined();
    expect(screen.getByText(/signal from Writer/)).toBeDefined();
  });

  test("editing one opens on its signal", async () => {
    const { fake, scout } = await openRoutines();
    act(() => {
      fake.routines.handlers()["routines.create"]({
        botId: scout.id,
        name: "Review",
        prompt: "Review the report",
        schedule: { kind: "signal", name: "report-ready" },
        timezone: "UTC",
      });
    });
    fireEvent.click(await screen.findByRole("button", { name: "More actions for Review" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Edit" }));
    const dialog = screen.getByRole("dialog", { name: "Edit Review" });
    expect(within(dialog).getByRole("radio", { name: "When a bot signals" })).toHaveProperty(
      "checked",
      true,
    );
    expect(within(dialog).getByLabelText("Signal")).toHaveProperty("value", "report-ready");
  });
});
