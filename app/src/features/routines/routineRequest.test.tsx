import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { ROUTINE_TOOL } from "../../lib/fakeChat";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

const INBOX = {
  name: "Morning inbox",
  prompt: "Read the new mail and summarize what is urgent.",
  schedule: { kind: "weekly", days: [1, 2, 3, 4, 5], time: "08:00" },
  timezone: "America/Sao_Paulo",
};

/** A crew with its chief and a bot "Mail", open on the chief's chat. */
async function openChief() {
  const fake = new FakeBotloft();
  const crew = await fake.call("crews.create", {
    name: "Office",
    lead: { name: "Chief", role: "Leads the crew", instructions: "Run the office" },
  });
  const chiefId = crew.leadBotId as string;
  const mail = await fake.call("bots.create", {
    crewId: crew.id,
    name: "Mail",
    role: "Reads the inbox",
    instructions: "",
  });
  fake.setBotState(chiefId, "idle");
  renderApp(fake);
  await crewOpened("Office");
  openBot("Chief");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, chiefId, mail };
}

function ask(fake: FakeBotloft, botId: string, input: object, title: string) {
  act(() => {
    fake.chat.ask(botId, ROUTINE_TOOL, "Morning inbox", JSON.stringify(input));
  });
  return screen.getByRole("region", { name: title });
}

const answer = (fake: FakeBotloft) =>
  fake.calls.find((call) => call.method === "approvals.answer")?.params;

const routines = (fake: FakeBotloft) => [...fake.routines.routines.values()];

describe("a bot asking for a routine", () => {
  test("the owner creates it as asked", async () => {
    const { fake, chiefId } = await openChief();
    const card = ask(fake, chiefId, INBOX, "Chief wants to set up a routine");
    expect((within(card).getByLabelText("Name") as HTMLInputElement).value).toBe("Morning inbox");
    expect(within(card).getByRole("radio", { name: "Weekdays" })).toHaveProperty("checked", true);
    fireEvent.click(within(card).getByRole("button", { name: "Create routine" }));

    expect(await screen.findByText("You created the routine Morning inbox")).toBeDefined();
    expect(answer(fake)).toEqual({ approvalId: expect.any(String), allow: true });
    expect(routines(fake)).toMatchObject([
      { botId: chiefId, name: "Morning inbox", timezone: "America/Sao_Paulo" },
    ]);
  });

  test("the owner can change when it runs before creating it", async () => {
    const { fake, chiefId } = await openChief();
    const card = ask(fake, chiefId, INBOX, "Chief wants to set up a routine");
    fireEvent.change(within(card).getByLabelText("Name"), { target: { value: "Inbox at nine" } });
    fireEvent.click(within(card).getByRole("radio", { name: "Every day" }));
    fireEvent.change(within(card).getByLabelText("At"), { target: { value: "09:00" } });
    fireEvent.click(within(card).getByRole("button", { name: "Create routine" }));

    expect(await screen.findByText("You created the routine Inbox at nine")).toBeDefined();
    const sent = JSON.parse(String((answer(fake) as { input?: string } | undefined)?.input));
    expect(sent).toMatchObject({
      name: "Inbox at nine",
      schedule: { kind: "weekly", days: [1, 2, 3, 4, 5, 6, 7], time: "09:00" },
    });
    expect(routines(fake)[0]?.schedule).toEqual(sent.schedule);
  });

  test("a routine for another bot of the crew goes to that bot", async () => {
    const { fake, chiefId, mail } = await openChief();
    const card = ask(
      fake,
      chiefId,
      { ...INBOX, bot: mail.handle },
      "Chief wants to set up a routine for Mail",
    );
    expect(within(card).getByLabelText("What should Mail do?")).toBeDefined();
    fireEvent.click(within(card).getByRole("button", { name: "Create routine" }));

    await screen.findByText("You created the routine Morning inbox");
    expect(routines(fake)).toMatchObject([{ botId: mail.id }]);
  });

  test("a no goes back with the owner's note and creates nothing", async () => {
    const { fake, chiefId } = await openChief();
    const card = ask(fake, chiefId, INBOX, "Chief wants to set up a routine");
    fireEvent.change(within(card).getByPlaceholderText(/If you say no, tell Chief why/), {
      target: { value: "I check it myself" },
    });
    fireEvent.click(within(card).getByRole("button", { name: "Not now" }));

    expect(await screen.findByText("You said no to the routine Morning inbox")).toBeDefined();
    expect(answer(fake)).toMatchObject({ allow: false, note: "I check it myself" });
    expect(routines(fake)).toEqual([]);
  });
});
