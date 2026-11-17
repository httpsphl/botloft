import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";
import { START_PLAN_TOOL } from "./StartPlanCard";

afterEach(cleanup);

const PLAN = {
  summary: "Rewrite the sourdough guide",
  steps: ["Read the guide", "Write five parts", "Add the photos"],
  alternative: "Only fix the starter part",
};

async function openWriter() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const writer = fake.addBot(ops.id, "Writer", "Writes posts");
  fake.setBotState(writer.id, "busy");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Writer");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, writer };
}

function show(fake: FakeBotloft, bot: string, plan: object, startsIn = 20_000) {
  act(() => {
    fake.chat.ask(
      bot,
      START_PLAN_TOOL,
      PLAN.summary,
      JSON.stringify({ ...plan, starts_at: Date.now() + startsIn }),
    );
  });
  return screen.getByRole("region", { name: "Writer will start like this" });
}

const answered = (fake: FakeBotloft) =>
  fake.calls.find((call) => call.method === "approvals.answer")?.params as
    | { allow: boolean; note?: string; input?: string }
    | undefined;

describe("the plan an agent shows before long work", () => {
  test("shows the steps, the smaller way and when it starts, and starts now", async () => {
    const { fake, writer } = await openWriter();
    const card = show(fake, writer.id, PLAN);
    expect(within(card).getByText("Rewrite the sourdough guide")).toBeDefined();
    expect(within(card).getAllByRole("listitem")).toHaveLength(3);
    expect(within(card).getByText("Only fix the starter part")).toBeDefined();
    expect(within(card).getByText(/Starts by itself in (19|20) s/)).toBeDefined();
    fireEvent.click(within(card).getByRole("button", { name: "Start now" }));
    await waitFor(() => expect(answered(fake)).toMatchObject({ allow: true }));
    expect(answered(fake)?.input).toBeUndefined();
    expect(await screen.findByText("You said start")).toBeDefined();
  });

  test("the smaller way is picked and kept with the plan", async () => {
    const { fake, writer } = await openWriter();
    const card = show(fake, writer.id, PLAN);
    fireEvent.click(within(card).getByRole("button", { name: "Do only this" }));
    await waitFor(() =>
      expect(answered(fake)).toMatchObject({ allow: true, input: '{"choice":"alternative"}' }),
    );
    expect(await screen.findByText("Doing only the smaller way")).toBeDefined();
  });

  test("a change goes with the owner's words, and a plan without a smaller way has no such button", async () => {
    const { fake, writer } = await openWriter();
    const card = show(fake, writer.id, { ...PLAN, alternative: null });
    expect(within(card).queryByRole("button", { name: "Do only this" })).toBeNull();
    fireEvent.click(within(card).getByRole("button", { name: "Change" }));
    const send = within(card).getByRole("button", { name: "Send" });
    expect(send).toHaveProperty("disabled", true);
    fireEvent.change(within(card).getByLabelText("What Writer should do instead"), {
      target: { value: "Start with the photos" },
    });
    fireEvent.click(send);
    await waitFor(() =>
      expect(answered(fake)).toMatchObject({ allow: false, note: "Start with the photos" }),
    );
    expect(await screen.findByText("You asked for changes")).toBeDefined();
  });

  test("cancel sends no note, and a plan past its time says it is starting", async () => {
    const { fake, writer } = await openWriter();
    const card = show(fake, writer.id, PLAN, -1000);
    expect(within(card).getByText("Starting…")).toBeDefined();
    fireEvent.click(within(card).getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(answered(fake)).toMatchObject({ allow: false }));
    expect(answered(fake)?.note).toBeUndefined();
    expect(await screen.findByText("You cancelled it")).toBeDefined();
  });

  test("the option is a switch in the agent's edit dialog", async () => {
    const { fake, writer } = await openWriter();
    expect(fake.bot(writer.id).planFirst).toBe(false);
    fireEvent.contextMenu(within(sidebar()).getByRole("button", { name: /^Writer,/ }), {
      clientX: 120,
      clientY: 200,
    });
    const menu = await screen.findByRole("menu", { name: "Actions for Writer" });
    fireEvent.click(within(menu).getByRole("menuitem", { name: "Edit" }));
    const dialog = within(await screen.findByRole("dialog", { name: /Edit Writer/ }));
    fireEvent.click(dialog.getByRole("switch", { name: "Show the plan before long tasks" }));
    fireEvent.click(dialog.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(fake.bot(writer.id).planFirst).toBe(true));
  });
});
