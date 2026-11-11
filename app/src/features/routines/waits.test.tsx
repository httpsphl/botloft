import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { en } from "../../i18n/en";
import { ptBR } from "../../i18n/pt-BR";
import { FakeBotloft } from "../../lib/fake";
import type { Schedule, WaitOn } from "../../lib/protocol.gen";
import { crewOpened, openBot, openTab, renderApp } from "../../test/app";
import { describeSchedule, waitWhat } from "./describe";

afterEach(cleanup);

const HOUR = 3_600_000;

/** Scout waiting for `on` (spec 20.14), as `wait_for` leaves it. */
function waiting(on: WaitOn) {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  const schedule: Schedule = { kind: "wait", on, until: Date.now() + 3 * HOUR };
  const routine = fake.routines.handlers()["routines.create"]({
    botId: scout.id,
    name: "Wait for the report",
    prompt: "Review the report.",
    schedule,
    timezone: "UTC",
  });
  return { fake, routine };
}

const file: WaitOn = { what: "file", path: "C:\\Botloft\\ops\\shared\\report.pdf" };

describe("an agent's wait", () => {
  test("shows over its message field with what and until when, and the owner cancels it", async () => {
    const { fake, routine } = waiting(file);
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });
    const pills = screen.getByRole("list", { name: "When Scout comes back to its work" });
    expect(
      // Three hours on may be tomorrow in UTC, the routine's zone.
      within(pills).getByText(/^Scout waits for report\.pdf to appear, until (today|tomorrow) at /),
    ).toBeDefined();
    fireEvent.click(within(pills).getByRole("button", { name: "Cancel: Wait for the report" }));
    await waitFor(() =>
      expect(fake.calls).toContainEqual({
        method: "routines.archive",
        params: { routineId: routine.id },
      }),
    );
  });

  test("reads as a routine without Edit or Run now", async () => {
    const { fake } = waiting(file);
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    openTab(/^Routines/);
    const row = (await screen.findByText("Wait for the report")).closest("li");
    if (!row) {
      throw new Error("no row");
    }
    expect(within(row).getByText("When report.pdf appears")).toBeDefined();
    expect(within(row).getByText(/Until (today|tomorrow) at /)).toBeDefined();
    expect(within(row).queryByRole("button", { name: "Run now" })).toBeNull();
    fireEvent.click(
      within(row).getByRole("button", { name: "More actions for Wait for the report" }),
    );
    expect(screen.queryByRole("menuitem", { name: "Edit" })).toBeNull();
    expect(screen.getByRole("menuitem", { name: "Delete" })).toBeDefined();
  });

  test("says what it waits for in each kind and language", () => {
    const page: WaitOn = { what: "page", url: "https://shop.test/order/1", text: "Enviado" };
    const program: WaitOn = { what: "program", name: "ffmpeg" };
    const until = 0;
    const when = (on: WaitOn) => describeSchedule({ kind: "wait", on, until }, en.routines.when);
    expect(when(file)).toBe("When report.pdf appears");
    expect(when(program)).toBe("When ffmpeg ends");
    expect(when(page)).toBe("When “Enviado” shows on shop.test");
    expect(waitWhat(program, ptBR.routines.reminder.wait)).toBe("ffmpeg terminar");
    expect(waitWhat(page, ptBR.routines.reminder.wait)).toBe("“Enviado” em shop.test");
    expect(
      describeSchedule(
        { kind: "wait", on: { what: "file", path: "/srv/out.csv" }, until },
        ptBR.routines.when,
      ),
    ).toBe("Quando out.csv aparecer");
  });
});
