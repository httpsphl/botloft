import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { en } from "../../i18n/en";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
import type { Routine } from "../../lib/protocol.gen";
import { RpcError } from "../../lib/rpc";
import { crewOpened, openBot, openTab, renderApp } from "../../test/app";
import { describeSchedule } from "./describe";

afterEach(cleanup);

/** A crew "Ops" with @scout, open on Scout's routines. */
async function openRoutines(setup?: (fake: FakeBotloft, botId: string) => void) {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  setup?.(fake, scout.id);
  const host = new FakeHost();
  renderApp(fake, host);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  openTab(/^Routines/);
  return { fake, scout };
}

function routine(fake: FakeBotloft, botId: string, name = "Morning"): Routine {
  return fake.routines.handlers()["routines.create"]({
    botId,
    name,
    prompt: "Summarize",
    schedule: { kind: "weekly", days: [1, 2, 3, 4, 5], time: "09:00" },
    timezone: "UTC",
  });
}

const created = (fake: FakeBotloft) =>
  fake.calls.find((call) => call.method === "routines.create")?.params;

function newRoutine(name: string) {
  fireEvent.click(screen.getByRole("button", { name: "New routine" }));
  const dialog = screen.getByRole("dialog", { name: "New routine for Scout" });
  fireEvent.change(within(dialog).getByLabelText("Name"), { target: { value: name } });
  fireEvent.change(within(dialog).getByLabelText("What should Scout do?"), {
    target: { value: "Summarize what arrived" },
  });
  return dialog;
}

describe("the schedule in words", () => {
  test("weekly, intervals and cron read plainly", () => {
    const words = en.routines.when;
    const weekly = (days: number[]) =>
      describeSchedule({ kind: "weekly", days, time: "09:00" }, words);
    expect(weekly([1, 2, 3, 4, 5])).toMatch(/^Weekdays at 09:00/);
    expect(weekly([1, 2, 3, 4, 5, 6, 7])).toMatch(/^Every day at/);
    expect(weekly([4, 1])).toMatch(/^Monday and Thursday at/);
    expect(describeSchedule({ kind: "interval", minutes: 120 }, words)).toBe("Every 2 hours");
    expect(describeSchedule({ kind: "interval", minutes: 30 }, words)).toBe("Every 30 minutes");
    expect(describeSchedule({ kind: "cron", expr: "0 9 * * 1" }, words)).toBe("Cron: 0 9 * * 1");
  });
});

describe("routines", () => {
  test("an agent with none says what a routine is", async () => {
    await openRoutines();
    expect(screen.getByText("Scout has no routines yet.")).toBeDefined();
  });

  test("the owner creates one for weekdays at nine", async () => {
    const { fake, scout } = await openRoutines();
    const dialog = newRoutine("Morning summary");
    fireEvent.click(within(dialog).getByRole("button", { name: "Create routine" }));

    await screen.findByText("Morning summary");
    expect(created(fake)).toMatchObject({
      botId: scout.id,
      name: "Morning summary",
      prompt: "Summarize what arrived",
      schedule: { kind: "weekly", days: [1, 2, 3, 4, 5], time: "09:00" },
      overlap: "skip",
      missed: "run_once",
    });
    expect(screen.getByText(/Weekdays at 09:00/)).toBeDefined();
    expect(screen.getByRole("tab", { name: "Routines (1)" })).toBeDefined();
  });

  // Many steps through the form: slow when the whole suite runs at once.
  test("chosen days, intervals and cron become their schedules", async () => {
    const { fake } = await openRoutines();
    let dialog = newRoutine("Two days");
    fireEvent.click(within(dialog).getByRole("radio", { name: "Chosen days" }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Thursday" }));
    fireEvent.change(within(dialog).getByLabelText("At"), { target: { value: "14:30" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Create routine" }));
    await screen.findByText("Two days");
    expect(created(fake)).toMatchObject({
      schedule: { kind: "weekly", days: [1, 4], time: "14:30" },
    });

    dialog = newRoutine("Every two hours");
    fireEvent.click(within(dialog).getByRole("radio", { name: "Every…" }));
    fireEvent.change(within(dialog).getByRole("spinbutton"), { target: { value: "2" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Create routine" }));
    await screen.findByText("Every two hours");
    const interval = fake.calls.filter((call) => call.method === "routines.create").at(-1);
    expect(interval?.params).toMatchObject({ schedule: { kind: "interval", minutes: 120 } });

    dialog = newRoutine("Nightly");
    fireEvent.click(within(dialog).getByText("More options"));
    fireEvent.click(within(dialog).getByLabelText("Use a cron expression"));
    fireEvent.change(within(dialog).getByLabelText("Cron expression"), {
      target: { value: "0 2 * * *" },
    });
    fireEvent.click(within(dialog).getByRole("radio", { name: "Wait its turn" }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Create routine" }));
    await screen.findByText("Nightly");
    const cron = fake.calls.filter((call) => call.method === "routines.create").at(-1);
    expect(cron?.params).toMatchObject({
      schedule: { kind: "cron", expr: "0 2 * * *" },
      overlap: "queue",
    });
  }, 15_000);

  test("a refusal the daemon explains is worded in the owner's language", async () => {
    const { fake } = await openRoutines();
    fake.failNext(
      "routines.create",
      new RpcError(-32004, "runs must be at least 5 minutes apart", "too_often"),
    );
    const dialog = newRoutine("Too often");
    fireEvent.click(within(dialog).getByRole("button", { name: "Create routine" }));
    expect(await within(dialog).findByText("Runs must be at least 5 minutes apart.")).toBeDefined();
  });

  test("the owner turns it off, runs it now, edits and deletes it", async () => {
    const { fake } = await openRoutines((fake, botId) => {
      routine(fake, botId);
    });
    const list = screen.getByRole("list", { name: "Routines" });
    const toggle = within(list).getByRole("switch", { name: "Morning on or off" });
    fireEvent.click(toggle);
    expect(await within(list).findByText(/· Off/)).toBeDefined();
    expect(toggle.getAttribute("aria-checked")).toBe("false");

    fireEvent.click(within(list).getByRole("button", { name: "Run now" }));
    expect(await within(list).findByText("Running now")).toBeDefined();

    fireEvent.click(within(list).getByRole("button", { name: "More actions for Morning" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Edit" }));
    const dialog = screen.getByRole("dialog", { name: "Edit Morning" });
    expect((within(dialog).getByLabelText("Name") as HTMLInputElement).value).toBe("Morning");
    expect(
      within(dialog).getByRole("radio", { name: "Weekdays" }).getAttribute("checked"),
    ).not.toBeNull();
    fireEvent.change(within(dialog).getByLabelText("Name"), { target: { value: "Dawn" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    expect(await within(list).findByText("Dawn")).toBeDefined();

    fireEvent.click(within(list).getByRole("button", { name: "More actions for Dawn" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Delete" }));
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(await screen.findByText("Scout has no routines yet.")).toBeDefined();
    expect(fake.calls.some((call) => call.method === "routines.archive")).toBe(true);
  });

  test("a routine's message shows its name in the chat", async () => {
    const { fake } = await openRoutines((fake, botId) => {
      routine(fake, botId, "Morning summary");
    });
    const [id] = [...fake.routines.routines.keys()];
    act(() => {
      void fake.call("routines.runNow", { routineId: id as string });
    });
    openTab("Chat");
    // Only its name, until the owner opens what it asks.
    const tag = await screen.findByRole("button", { name: "Routine · Morning summary" });
    const prompt = fake.routines.routines.get(id as string)?.prompt as string;
    const chat = screen.getByRole("list", { name: "Messages" });
    expect(within(chat).queryByText(prompt)).toBeNull();
    fireEvent.click(tag);
    expect(tag.getAttribute("aria-expanded")).toBe("true");
    expect(within(chat).getByText(prompt)).toBeDefined();
  });

  test("the crew's page lists every agent's routines", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout");
    const writer = fake.addBot(ops.id, "Writer");
    routine(fake, scout.id, "Morning");
    routine(fake, writer.id, "Draft");
    renderApp(fake);
    await crewOpened("Ops");
    openTab("Routines");
    const list = await screen.findByRole("list", { name: "Routines" });
    expect(within(list).getByText("Morning")).toBeDefined();
    expect(within(list).getByText("Draft")).toBeDefined();
  });
});

describe("failed runs", () => {
  test("mark the taskbar until the owner opens the agent", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout");
    fake.addBot(ops.id, "Writer");
    const morning = routine(fake, scout.id);
    const host = new FakeHost();
    renderApp(fake, host);
    await crewOpened("Ops");
    openBot("Writer");
    act(() => {
      fake.emit({
        name: "routine.run",
        params: {
          id: "rrn_9999",
          routineId: morning.id,
          scheduledFor: Date.now(),
          status: "failed",
          reason: null,
          skippedCount: 0,
          messageId: null,
          createdAt: Date.now(),
          finishedAt: Date.now() + 1000,
          signal: null,
        },
      });
    });
    await act(() => Promise.resolve());
    expect(host.attention).toBe(true);
    openBot("Scout");
    await screen.findByRole("heading", { level: 1, name: "Scout" });
    await act(() => Promise.resolve());
    expect(host.attention).toBe(false);
  });
});
