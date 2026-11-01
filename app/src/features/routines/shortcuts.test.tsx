import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { en } from "../../i18n/en";
import { FakeBotloft } from "../../lib/fake";
import type { Schedule } from "../../lib/protocol.gen";
import { crewOpened, openBot, openTab, renderApp } from "../../test/app";
import { describeSchedule } from "./describe";

afterEach(cleanup);

function crew(schedule: Schedule, name = "Morning summary") {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  const routine = fake.routines.handlers()["routines.create"]({
    botId: scout.id,
    name,
    prompt: "Summarize what arrived",
    schedule,
    timezone: "UTC",
  });
  return { fake, scout, routine };
}

const ran = (fake: FakeBotloft) => fake.calls.filter((call) => call.method === "routines.runNow");

describe("a shortcut", () => {
  test("is a button on the home that runs at once and says who took it", async () => {
    const { fake, routine } = crew({ kind: "manual" });
    renderApp(fake);
    const shortcuts = await screen.findByRole("region", { name: "Shortcuts" });
    fireEvent.click(
      within(shortcuts).getByRole("button", { name: "Run Morning summary, by Scout" }),
    );
    await waitFor(() => expect(ran(fake)).toHaveLength(1));
    expect(ran(fake)[0]?.params).toEqual({ routineId: routine.id });
    expect(await screen.findByText("Scout started: Morning summary")).toBeDefined();
  });

  test("is in the agent's chat by its message field", async () => {
    const { fake } = crew({ kind: "manual" });
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });
    fireEvent.click(screen.getByRole("button", { name: "Scout's shortcuts" }));
    const menu = screen.getByRole("menu", { name: "Scout's shortcuts" });
    fireEvent.click(within(menu).getByRole("menuitem", { name: "Morning summary" }));
    await waitFor(() => expect(ran(fake)).toHaveLength(1));
  });

  test("does not show without one, and says when it runs", async () => {
    const { fake } = crew({ kind: "weekly", days: [1], time: "09:00" });
    renderApp(fake);
    await screen.findByRole("heading", { name: "Home" });
    expect(screen.queryByRole("region", { name: "Shortcuts" })).toBeNull();
    const words = en.routines.when;
    expect(describeSchedule({ kind: "manual" }, words)).toBe("When you click it");
    expect(describeSchedule({ kind: "once", at: "2026-11-03T15:30" }, words)).toMatch(
      /^Once, Nov 3 at /,
    );
  });
});

describe("an agent's own reminder", () => {
  test("shows over its message field, and the owner cancels it", async () => {
    const at = new Date(Date.now() + 2 * 3_600_000).toISOString().slice(0, 16);
    const { fake, routine } = crew({ kind: "once", at, reminder: true }, "Look at the deploy");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });
    const reminders = screen.getByRole("list", { name: "When Scout comes back to its work" });
    expect(within(reminders).getByText(/^Scout comes back to it /)).toBeDefined();
    fireEvent.click(within(reminders).getByRole("button", { name: "Cancel: Look at the deploy" }));
    await waitFor(() =>
      expect(fake.calls).toContainEqual({
        method: "routines.archive",
        params: { routineId: routine.id },
      }),
    );
  });
});

describe("a one-time routine", () => {
  test("is made with a date and a time", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "idle");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });
    openTab(/^Routines/);
    fireEvent.click(screen.getByRole("button", { name: "New routine" }));
    const dialog = screen.getByRole("dialog", { name: "New routine for Scout" });
    fireEvent.change(within(dialog).getByLabelText("Name"), { target: { value: "Report" } });
    fireEvent.change(within(dialog).getByLabelText("What should Scout do?"), {
      target: { value: "Write the report" },
    });
    fireEvent.click(within(dialog).getByRole("radio", { name: "Once" }));
    fireEvent.change(within(dialog).getByLabelText("On"), { target: { value: "2030-01-02" } });
    fireEvent.change(within(dialog).getByLabelText("At"), { target: { value: "15:30" } });
    expect(within(dialog).getByText(/stays as a shortcut/)).toBeDefined();
    act(() => {
      fireEvent.click(within(dialog).getByRole("button", { name: "Create routine" }));
    });
    await waitFor(() =>
      expect(fake.calls.find((call) => call.method === "routines.create")?.params).toMatchObject({
        schedule: { kind: "once", at: "2030-01-02T15:30" },
      }),
    );
  });
});
