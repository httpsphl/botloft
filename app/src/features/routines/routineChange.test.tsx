import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { CHANGE_ROUTINE_TOOL, DELETE_ROUTINE_TOOL } from "./RoutineChangeCard";

afterEach(cleanup);

const BEFORE = {
  routine_id: "rtn_1",
  name: "Morning inbox",
  prompt: "Read the new mail.",
  schedule: { kind: "weekly", days: [1, 2, 3, 4, 5], time: "08:00" },
  timezone: "America/Sao_Paulo",
  overlap: "skip",
  missed: "run_once",
  enabled: true,
};

async function openMail() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Office");
  const mail = fake.addBot(ops.id, "Mail", "Reads the inbox");
  fake.setBotState(mail.id, "idle");
  renderApp(fake);
  await crewOpened("Office");
  openBot("Mail");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, mail };
}

const answered = (fake: FakeBotloft) =>
  fake.calls.find((call) => call.method === "approvals.answer")?.params as
    | { allow: boolean; input?: string; note?: string }
    | undefined;

describe("an agent changing or deleting its routine", () => {
  test("the change shows before and after, and goes as asked", async () => {
    const { fake, mail } = await openMail();
    const after = {
      ...BEFORE,
      name: "Inbox",
      schedule: { kind: "interval", minutes: 120 },
      enabled: false,
    };
    act(() => {
      fake.chat.ask(
        mail.id,
        CHANGE_ROUTINE_TOOL,
        "Morning inbox",
        JSON.stringify({ name: "Morning inbox", before: BEFORE, after }),
      );
    });
    const card = screen.getByRole("region", {
      name: "Mail wants to change the routine Morning inbox",
    });
    expect(within(card).getByText(/^Weekdays at 08:00/)).toBeDefined();
    expect(within(card).getByText("Every 2 hours")).toBeDefined();
    expect(within(card).getByText("Off")).toBeDefined();
    fireEvent.click(within(card).getByRole("button", { name: "Change routine" }));
    await waitFor(() => expect(answered(fake)).toMatchObject({ allow: true }));
    expect(answered(fake)?.input).toBeUndefined();
  });

  test("the owner adjusts the change before allowing it", async () => {
    const { fake, mail } = await openMail();
    act(() => {
      fake.chat.ask(
        mail.id,
        CHANGE_ROUTINE_TOOL,
        "Morning inbox",
        JSON.stringify({
          name: "Morning inbox",
          before: BEFORE,
          after: { ...BEFORE, enabled: false },
        }),
      );
    });
    const card = screen.getByRole("region", { name: /wants to change/ });
    fireEvent.click(within(card).getByRole("button", { name: "Adjust before allowing" }));
    fireEvent.change(within(card).getByLabelText("Name"), { target: { value: "Inbox at eight" } });
    fireEvent.click(within(card).getByRole("button", { name: "Change routine" }));
    await waitFor(() => expect(answered(fake)?.input).toBeDefined());
    const input = JSON.parse(answered(fake)?.input as string);
    expect(input).toMatchObject({ routine_id: "rtn_1", name: "Inbox at eight", enabled: false });
  });

  test("a deletion shows why, and the owner may keep it with a note", async () => {
    const { fake, mail } = await openMail();
    act(() => {
      fake.chat.ask(
        mail.id,
        DELETE_ROUTINE_TOOL,
        "Morning inbox",
        JSON.stringify({
          routine_id: "rtn_1",
          name: "Morning inbox",
          schedule: BEFORE.schedule,
          reason: "The client left.",
        }),
      );
    });
    const card = screen.getByRole("region", {
      name: "Mail wants to delete the routine Morning inbox",
    });
    expect(within(card).getByText("The client left.")).toBeDefined();
    fireEvent.change(within(card).getByRole("textbox"), { target: { value: "Keep it" } });
    fireEvent.click(within(card).getByRole("button", { name: "Keep it" }));
    await waitFor(() => expect(answered(fake)).toMatchObject({ allow: false, note: "Keep it" }));
  });
});
