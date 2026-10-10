import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, renderApp, sidebar } from "../../test/app";

afterEach(cleanup);

describe("the first screen for someone who does not use Claude Code", () => {
  test("Claude Code missing is not a blocker when another agent is ready", async () => {
    const fake = new FakeBotloft();
    fake.system = {
      ...fake.system,
      claudeVersion: null,
      runtimeError: "Claude Code was not found",
      enabledAgents: ["claude", "agy"],
      agentChecks: [{ agent: "agy", version: "1.3.1" }],
    };
    renderApp(fake);
    expect(await screen.findByText("Antigravity (experimental)")).toBeDefined();
    expect(screen.getByText("Version 1.3.1")).toBeDefined();
    expect(screen.getByText(/only need it for agents that run on Claude Code/)).toBeDefined();
    expect(screen.queryByText(/Install Claude Code/i)).toBeNull();
  });

  test("an agent that is not found says so", async () => {
    const fake = new FakeBotloft();
    fake.system = {
      ...fake.system,
      enabledAgents: ["claude", "agy"],
      agentChecks: [{ agent: "agy", version: null }],
    };
    renderApp(fake);
    expect(await screen.findByText(/Not found\. Install it/)).toBeDefined();
  });
});

describe("the agent for new agents", () => {
  test("the new-bot dialog starts on the owner's choice", async () => {
    const fake = new FakeBotloft();
    fake.system = { ...fake.system, enabledAgents: ["claude", "agy"] };
    fake.settings = { ...fake.settings, defaultAgent: "agy" };
    fake.addCrew("Ops");
    renderApp(fake);
    await crewOpened("Ops");
    const [button] = await screen.findAllByRole("button", { name: "New agent" });
    fireEvent.click(button as HTMLElement);
    const dialog = within(await screen.findByRole("dialog", { name: "New agent" }));
    expect((dialog.getByLabelText("Runs on") as HTMLSelectElement).value).toBe("agy");
  });

  test("full access is a switch over the command list", async () => {
    const fake = new FakeBotloft();
    fake.system = { ...fake.system, enabledAgents: ["claude", "agy"] };
    const crew = fake.addCrew("Ops");
    const bot = fake.addBot(crew.id, "Zed");
    fake.bot(bot.id).agent = "agy";
    fake.setBotState(bot.id, "idle");
    renderApp(fake);
    await crewOpened("Ops");
    fireEvent.contextMenu(within(sidebar()).getByRole("button", { name: /^Zed,/ }), {
      clientX: 100,
      clientY: 100,
    });
    fireEvent.click(
      within(await screen.findByRole("menu", { name: "Actions for Zed" })).getByRole("menuitem", {
        name: "Edit",
      }),
    );
    const dialog = within(await screen.findByRole("dialog", { name: /Edit Zed/ }));
    fireEvent.click(dialog.getByRole("switch", { name: "Run any command without asking" }));
    expect(dialog.queryByLabelText("Commands this agent may run")).toBeNull();
    fireEvent.click(dialog.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(fake.bot(bot.id).allowedCommands).toEqual(["*"]));
  });
});
