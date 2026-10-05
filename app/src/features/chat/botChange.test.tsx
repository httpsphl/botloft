import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { CHANGE_BOT_TOOL } from "./BotChangeCard";

afterEach(cleanup);

const WRITER = {
  name: "Writer",
  role: "Writes posts",
  instructions: "Short sentences.",
  model: "sonnet",
  effort: "default",
};

async function openChief() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const chief = fake.addBot(ops.id, "Chief", "Leads the crew");
  fake.setBotState(chief.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Chief");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, chief };
}

const answered = (fake: FakeBotloft) =>
  fake.calls.find((call) => call.method === "approvals.answer")?.params as
    | { allow: boolean; note?: string }
    | undefined;

describe("a bot asking to change a bot", () => {
  test("the chief's request shows the other bot, before and after, and why", async () => {
    const { fake, chief } = await openChief();
    act(() => {
      fake.chat.ask(
        chief.id,
        CHANGE_BOT_TOOL,
        "Writer",
        JSON.stringify({
          bot_id: "bot_writer",
          name: "Writer",
          before: WRITER,
          after: {
            ...WRITER,
            name: "Editor",
            instructions: "Edit, do not write.",
            model: "haiku",
            effort: "low",
          },
          reason: "The owner asked for it.",
        }),
      );
    });
    const card = screen.getByRole("region", { name: "Chief wants to change Writer" });
    expect(within(card).getByText("Editor")).toBeDefined();
    expect(within(card).getByText("Sonnet")).toBeDefined();
    expect(within(card).getByText("Haiku")).toBeDefined();
    expect(within(card).getByText("Recommended")).toBeDefined();
    expect(within(card).getByText("Low")).toBeDefined();
    expect(within(card).queryByText("Writes posts")).toBeNull();
    expect(within(card).getByText("See the new instructions")).toBeDefined();
    expect(within(card).getByText("The owner asked for it.")).toBeDefined();
    fireEvent.click(within(card).getByRole("button", { name: "Change" }));
    await waitFor(() => expect(answered(fake)).toMatchObject({ allow: true }));
  });

  test("a bot changing itself, declined with a note", async () => {
    const { fake, chief } = await openChief();
    act(() => {
      fake.chat.ask(
        chief.id,
        CHANGE_BOT_TOOL,
        "Chief",
        JSON.stringify({
          bot_id: chief.id,
          name: "Chief",
          before: { ...WRITER, name: "Chief" },
          after: { ...WRITER, name: "Boss" },
          reason: null,
        }),
      );
    });
    const card = screen.getByRole("region", { name: "Chief wants to change how it is set up" });
    fireEvent.change(within(card).getByRole("textbox"), { target: { value: "Keep the name" } });
    fireEvent.click(within(card).getByRole("button", { name: "Not now" }));
    await waitFor(() =>
      expect(answered(fake)).toMatchObject({ allow: false, note: "Keep the name" }),
    );
  });
});
