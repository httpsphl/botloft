import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { SUGGEST_TOOL } from "../../lib/fakeChat";
import { crewOpened, openBot, openNewCrewForm, renderApp, sidebar } from "../../test/app";

afterEach(cleanup);

const DESIGNER = {
  name: "Designer",
  role: "Draws each page",
  instructions: "Design the pages in shared/design.",
  model: "sonnet",
  reason: "The site needs a look before anyone builds it.",
};

/** A crew "Site" with its chief, open on the chief's chat. */
async function openChief() {
  const fake = new FakeBotloft();
  const site = fake.call("crews.create", {
    name: "Site",
    lead: { name: "Chief", role: "Leads the crew", instructions: "Build the site" },
  });
  const crew = await site;
  const chiefId = crew.leadBotId as string;
  fake.setBotState(chiefId, "idle");
  renderApp(fake);
  await crewOpened("Site");
  openBot("Chief");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, crew, chiefId };
}

function suggest(fake: FakeBotloft, chiefId: string) {
  act(() => {
    fake.chat.ask(chiefId, SUGGEST_TOOL, "Designer", JSON.stringify(DESIGNER));
  });
  return screen.getByRole("region", { name: "Chief suggests a new bot" });
}

const answer = (fake: FakeBotloft) =>
  fake.calls.find((call) => call.method === "approvals.answer")?.params;

describe("the crew's chief", () => {
  test("a new crew starts with a chief, from the crew's goal", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    renderApp(fake);
    await crewOpened("Ops");
    const dialog = await openNewCrewForm();
    fireEvent.change(within(dialog).getByLabelText("Name"), { target: { value: "Bakery" } });
    fireEvent.change(within(dialog).getByLabelText("What is this crew for?"), {
      target: { value: "Build my bakery's website" },
    });
    fireEvent.change(within(dialog).getByLabelText("Chief's model"), {
      target: { value: "opus" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Create crew" }));

    expect(await screen.findByRole("heading", { level: 1, name: "Chief" })).toBeDefined();
    const create = fake.calls.find((call) => call.method === "crews.create");
    expect(create?.params).toMatchObject({
      name: "Bakery",
      lead: { name: "Chief", instructions: "Build my bakery's website", model: "opus" },
    });
    // The crown, in the header and the conversation list.
    expect(screen.getAllByTitle(/^Leads Bakery/)).toHaveLength(2);
  });

  test("the owner creates the bot the chief suggests", async () => {
    const { fake, chiefId } = await openChief();
    const card = suggest(fake, chiefId);
    expect(within(card).getByText(DESIGNER.reason)).toBeDefined();
    expect((within(card).getByLabelText("Name") as HTMLInputElement).value).toBe("Designer");
    fireEvent.click(within(card).getByRole("button", { name: "Create bot" }));

    expect(await screen.findByText("You created Designer")).toBeDefined();
    expect(answer(fake)).toEqual({ approvalId: expect.any(String), allow: true });
    expect(within(sidebar()).getByRole("button", { name: /^Designer/ })).toBeDefined();
  });

  test("the owner can change the suggestion before creating it", async () => {
    const { fake, chiefId } = await openChief();
    const card = suggest(fake, chiefId);
    fireEvent.change(within(card).getByLabelText("Name"), { target: { value: "Illustrator" } });
    fireEvent.change(within(card).getByLabelText("Model"), { target: { value: "haiku" } });
    fireEvent.click(within(card).getByRole("button", { name: "Create bot" }));

    expect(await screen.findByText("You created Illustrator")).toBeDefined();
    const sent = answer(fake) as { input: string };
    expect(JSON.parse(sent.input)).toMatchObject({ name: "Illustrator", model: "haiku" });
    const illustrator = [...fake.bots.values()].find((bot) => bot.name === "Illustrator");
    expect(illustrator?.model).toBe("haiku");
  });

  test("saying no tells the chief why", async () => {
    const { fake, chiefId } = await openChief();
    const card = suggest(fake, chiefId);
    fireEvent.change(within(card).getByLabelText("What to tell Chief if you say no"), {
      target: { value: "The writer can do it" },
    });
    fireEvent.click(within(card).getByRole("button", { name: "Not now" }));

    expect(await screen.findByText("You said no to Designer")).toBeDefined();
    expect(answer(fake)).toMatchObject({ allow: false, note: "The writer can do it" });
    expect([...fake.bots.values()].some((bot) => bot.name === "Designer")).toBe(false);
  });

  test("the owner moves the chief to another bot", async () => {
    const { fake, crew } = await openChief();
    act(() => {
      fake.setBotState(fake.addBot(crew.id, "Writer").id, "idle");
    });
    openBot("Writer");
    await screen.findByRole("heading", { level: 1, name: "Writer" });
    fireEvent.click(screen.getByRole("button", { name: "More bot actions" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Make crew chief" }));

    // The header and the conversation list of the new chief.
    await screen.findByText("Chief", { selector: "header span" });
    const moved = fake.calls.find((call) => call.method === "crews.setLead");
    expect(moved?.params).toMatchObject({ crewId: crew.id });
    fireEvent.click(screen.getByRole("button", { name: "More bot actions" }));
    expect(screen.getByRole("menuitem", { name: "Stop being chief" })).toBeDefined();
  });
});
