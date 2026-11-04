import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { prefs } from "../../shell/prefs";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";
import { BUILD_TOOL } from "./plan";

afterEach(() => {
  cleanup();
  prefs.showMaker.reset();
});

const PLAN = {
  title: "A crew for the blog",
  why: "You write a post every week by hand.",
  steps: [
    {
      op: "create_crew",
      ref: "blog",
      name: "Blog",
      goal: "Publish one good post a week.",
      lead_name: "Chief",
      lead_role: "Leads the crew",
      memory: "Warm tone, no prices.",
    },
    {
      op: "add_agent",
      crew: "blog",
      name: "Writer",
      role: "Writes the posts",
      instructions: "Write.",
      model: "sonnet",
    },
    {
      op: "add_routine",
      crew: "blog",
      agent: "Writer",
      name: "Weekly post",
      prompt: "Write this week's post.",
      schedule: { kind: "weekly", days: [1], time: "08:00" },
      timezone: "UTC",
    },
  ],
};

/** The Maker's chat open, next to a crew of the owner's. */
async function openMaker() {
  const fake = new FakeBotloft();
  fake.addCrew("Ops");
  const maker = fake.maker.ensure();
  fake.setBotState(maker.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("^The Maker");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, maker };
}

function propose(fake: FakeBotloft, makerId: string) {
  act(() => {
    fake.chat.ask(makerId, BUILD_TOOL, PLAN.title, JSON.stringify(PLAN));
  });
  return screen.getByRole("region", { name: "The Maker wants to build: A crew for the blog" });
}

const sent = (fake: FakeBotloft) =>
  fake.calls.find((call) => call.method === "approvals.answer")?.params as {
    allow: boolean;
    input?: string;
  };

describe("the Maker", () => {
  test("shows as the white flame, with its wand and flow in its empty chat", async () => {
    const { maker } = await openMaker();
    const mascots = [...document.querySelectorAll<SVGSVGElement>(`svg[data-bot="${maker.id}"]`)];
    expect(mascots.length).toBeGreaterThan(1);
    expect(mascots.every((svg) => svg.hasAttribute("data-maker"))).toBe(true);
    expect(document.querySelector("svg[data-maker] .maker-flow")).not.toBeNull();
    expect(document.querySelector("svg[data-maker] .maker-wand")).not.toBeNull();
    // An agent of the owner's never looks like the Maker.
    const others = [...document.querySelectorAll("svg[data-bot]")].filter(
      (svg) => svg.getAttribute("data-bot") !== maker.id,
    );
    expect(others.some((svg) => svg.hasAttribute("data-maker"))).toBe(false);
  });

  test("sits above the crews, without its own crew in the list", async () => {
    const { fake } = await openMaker();
    expect(screen.getByText("Tell me what you do, and I build your crews")).toBeDefined();
    expect(within(sidebar()).queryByRole("button", { name: "Maker" })).toBeNull();
    act(() => prefs.showMaker.set(false));
    expect(within(sidebar()).queryByRole("button", { name: /^The Maker/ })).toBeNull();
    expect(fake.crews.size).toBe(2);
  });

  test("the owner builds its plan and can undo it", async () => {
    const { fake, maker } = await openMaker();
    const card = propose(fake, maker.id);
    expect(within(card).getByText(PLAN.why)).toBeDefined();
    expect(within(card).getByText("Chief, the chief")).toBeDefined();
    expect(within(card).getByText(/1 routine will run on its own/)).toBeDefined();
    fireEvent.click(within(card).getByRole("button", { name: "Build" }));

    expect(await screen.findByText("You built: A crew for the blog")).toBeDefined();
    expect(sent(fake)).toEqual({ approvalId: expect.any(String), allow: true });
    expect(await within(sidebar()).findByRole("button", { name: /^Blog/ })).toBeDefined();

    fireEvent.click(await screen.findByRole("button", { name: "Undo" }));
    const dialog = await screen.findByRole("dialog", { name: "Undo what the Maker built?" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Undo" }));
    expect(await screen.findByText("Undone: the crews are in the archive")).toBeDefined();
    expect(within(sidebar()).queryByRole("button", { name: /^Blog/ })).toBeNull();
  });

  test("leaving out an agent leaves out its routines", async () => {
    const { fake, maker } = await openMaker();
    const card = propose(fake, maker.id);
    fireEvent.click(within(card).getByRole("checkbox", { name: "Include Writer" }));
    expect(within(card).queryByText(/routine will run/)).toBeNull();
    fireEvent.click(within(card).getByRole("button", { name: "Build" }));

    expect(await screen.findByText("You built: A crew for the blog")).toBeDefined();
    const input = JSON.parse(sent(fake).input ?? "{}") as { steps: { op: string }[] };
    expect(input.steps.map((step) => step.op)).toEqual(["create_crew"]);
  });

  test("leaving out every crew leaves nothing to build", async () => {
    const { fake, maker } = await openMaker();
    const card = propose(fake, maker.id);
    fireEvent.click(within(card).getByRole("checkbox", { name: "Include Blog" }));
    expect(within(card).getByText("Leave at least one crew in the plan.")).toBeDefined();
    expect(
      (within(card).getByRole("button", { name: "Build" }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  test("before the first crew, the welcome opens the Maker's chat", async () => {
    const fake = new FakeBotloft();
    fake.maker.ensure();
    renderApp(fake);
    fireEvent.click(await screen.findByRole("button", { name: /Let the Maker build/ }));
    expect(await screen.findByText("Tell me what you do, and I build your crews")).toBeDefined();
  });
});

describe("the Maker's menu", () => {
  test("offers only what the Maker can do, and its edit keeps its name and white", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    fake.maker.ensure();
    renderApp(fake);
    await crewOpened("Ops");
    const row = within(sidebar()).getByRole("button", { name: /^The Maker/ });
    fireEvent.contextMenu(row, { clientX: 120, clientY: 200 });
    const menu = screen.getByRole("menu", { name: "Actions for The Maker" });
    const offered = within(menu)
      .getAllByRole("menuitem")
      .map((item) => item.textContent);
    expect(offered).toEqual([
      "Edit",
      "Bring my Claude Code history",
      "Bring my claude.ai export",
      "Restart",
      "Restart with a new conversation",
      "Open folder",
      "Mark as unread",
    ]);
    // A history comes from the menu too, once the chat is not empty.
    fireEvent.click(within(menu).getByRole("menuitem", { name: "Bring my Claude Code history" }));
    const history = await screen.findByRole("dialog", { name: "Bring your Claude Code history" });
    fireEvent.click(within(history).getByRole("button", { name: "Close" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());

    fireEvent.contextMenu(row, { clientX: 120, clientY: 200 });
    const again = screen.getByRole("menu", { name: "Actions for The Maker" });
    fireEvent.click(within(again).getByRole("menuitem", { name: "Edit" }));
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).queryByRole("textbox", { name: /^Name/ })).toBeNull();
    expect(within(dialog).queryByRole("button", { name: "Pick any color" })).toBeNull();
    expect(within(dialog).getByRole("textbox", { name: /Instructions/ })).toBeDefined();
  });
});
