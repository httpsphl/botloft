import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with @scout, open on its chat. */
async function openScout() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout };
}

const picker = () => screen.getByRole("button", { name: /^Mode: / });
const modeCalls = (fake: FakeBotloft) =>
  fake.calls.filter((call) => call.method === "bots.setPermissionMode");

function choose(name: RegExp) {
  fireEvent.click(picker());
  const menu = screen.getByRole("menu", { name: "Mode" });
  fireEvent.click(within(menu).getByRole("menuitemradio", { name: name }));
}

describe("permission modes", () => {
  test("an agent starts in Manual and the owner picks another mode", async () => {
    const { fake, scout } = await openScout();
    expect(picker().getAttribute("aria-label")).toBe("Mode: Manual");
    fireEvent.click(picker());
    const menu = screen.getByRole("menu", { name: "Mode" });
    const manual = within(menu).getByRole("menuitemradio", { name: /Manual/ });
    expect(manual.getAttribute("aria-checked")).toBe("true");
    fireEvent.click(within(menu).getByRole("menuitemradio", { name: /Accept edits/ }));

    await screen.findByRole("button", { name: "Mode: Accept edits" });
    expect(screen.queryByRole("menu", { name: "Mode" })).toBeNull();
    expect(modeCalls(fake).at(-1)?.params).toEqual({ botId: scout.id, mode: "accept_edits" });
    // Nothing was running: no note about waiting.
    expect(screen.queryByText(/switches to/)).toBeNull();
  });

  test("bypassing permissions asks first, then marks the agent in red", async () => {
    const { fake } = await openScout();
    choose(/Bypass permissions/);
    const dialog = screen.getByRole("dialog", { name: "Let Scout do anything without asking?" });
    expect(within(dialog).getByText(/other agents' files/)).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(modeCalls(fake)).toHaveLength(0);
    expect(screen.queryByText("Asks nothing")).toBeNull();

    choose(/Bypass permissions/);
    fireEvent.click(screen.getByRole("button", { name: "Turn on" }));
    await screen.findByRole("button", { name: "Mode: Bypass permissions" });
    expect(screen.getByText("Asks nothing")).toBeDefined();
    expect(modeCalls(fake).at(-1)?.params).toMatchObject({ mode: "bypass_permissions" });
  });

  test("a busy agent changes mode when it finishes what it is doing", async () => {
    const { fake, scout } = await openScout();
    act(() => fake.setBotState(scout.id, "busy"));
    choose(/Plan/);
    expect(
      await screen.findByText("Scout switches to Plan when it finishes what it's doing."),
    ).toBeDefined();
    act(() => fake.setBotState(scout.id, "idle"));
    expect(screen.queryByText(/switches to Plan/)).toBeNull();
  });
});

describe("plans", () => {
  const PLAN = "## Cache the report\n\n1. Save it once a week";

  function askToGoAhead(fake: FakeBotloft, botId: string) {
    act(() => {
      fake.chat.ask(botId, "ExitPlanMode", "Cache the report", JSON.stringify({ plan: PLAN }));
    });
    return screen.getByRole("region", { name: "Scout made a plan and wants to go ahead" });
  }

  test("the plan shows in full and approving it lets the agent go ahead", async () => {
    const { fake, scout } = await openScout();
    await act(() => fake.call("bots.setPermissionMode", { botId: scout.id, mode: "plan" }));
    const card = askToGoAhead(fake, scout.id);
    expect(within(card).getByRole("heading", { name: "Cache the report" })).toBeDefined();
    expect(within(card).getByText("Save it once a week")).toBeDefined();

    fireEvent.click(within(card).getByRole("button", { name: "Approve plan" }));
    expect(await screen.findByText("You approved the plan")).toBeDefined();
    const answer = fake.calls.find((call) => call.method === "approvals.answer");
    expect(answer?.params).toMatchObject({ allow: true });
    // As Claude Code does, the bot left plan mode.
    expect(await screen.findByRole("button", { name: "Mode: Manual" })).toBeDefined();
  });

  test("keeping on planning sends back what should change", async () => {
    const { fake, scout } = await openScout();
    const card = askToGoAhead(fake, scout.id);
    fireEvent.change(within(card).getByLabelText("What Scout should change in the plan"), {
      target: { value: "Keep the cache in memory" },
    });
    fireEvent.click(within(card).getByRole("button", { name: "Keep planning" }));
    expect(await screen.findByText("You asked for changes to the plan")).toBeDefined();
    expect(screen.getByText("“Keep the cache in memory”")).toBeDefined();
    const answer = fake.calls.find((call) => call.method === "approvals.answer");
    expect(answer?.params).toMatchObject({ allow: false, note: "Keep the cache in memory" });
  });
});
