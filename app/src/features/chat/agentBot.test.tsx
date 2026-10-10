import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";

afterEach(cleanup);

async function agyBot() {
  const fake = new FakeBotloft();
  const crew = fake.addCrew("Ops");
  const bot = fake.addBot(crew.id, "Zed");
  fake.bot(bot.id).agent = "agy";
  fake.setBotState(bot.id, "idle");
  fake.setContext(bot.id, {
    usedTokens: 120_000,
    windowTokens: 1_000_000,
    autoCompactTokens: null,
    compacting: false,
    updatedAt: 0,
  });
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Zed");
  await screen.findByRole("list", { name: "Messages" });
  return fake;
}

describe("an agent that runs on another agent", () => {
  test("shows its models, not Claude's, and picks one", async () => {
    const fake = await agyBot();
    expect(screen.queryByRole("button", { name: /^Effort:/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /Plan default|Opus|Sonnet/ })).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Model: Default" }));
    const menu = await screen.findByRole("menu", { name: "Model" });
    fireEvent.click(
      await within(menu).findByRole("menuitemradio", { name: /Gemini 3.8 Flash \(High\)/ }),
    );
    await waitFor(() =>
      expect(fake.bots.values().next().value?.agentModel).toBe("gemini-3.8-flash-high"),
    );
  });

  test("shows how full the conversation is, without a way to compact it", async () => {
    await agyBot();
    fireEvent.click(screen.getByRole("button", { name: /Conversation space/ }));
    const panel = await screen.findByRole("dialog", { name: "Conversation space" });
    expect(within(panel).getByText(/An estimate/)).toBeDefined();
    expect(within(panel).queryByRole("button", { name: "Compact now" })).toBeNull();
  });
});

describe("what an agent that cannot ask needs from the owner", () => {
  test("the commands it may run are a list in the agent's edit dialog", async () => {
    const fake = await agyBot();
    const bot = [...fake.bots.values()][0];
    expect(bot?.allowedCommands).toEqual([]);
    // The bot's right-click menu in the list opens its edit dialog.
    fireEvent.contextMenu(within(sidebar()).getByRole("button", { name: /^Zed,/ }), {
      clientX: 120,
      clientY: 200,
    });
    const menu = await screen.findByRole("menu", { name: "Actions for Zed" });
    fireEvent.click(within(menu).getByRole("menuitem", { name: "Edit" }));
    const dialog = within(await screen.findByRole("dialog", { name: /Edit Zed/ }));
    fireEvent.change(dialog.getByLabelText("Commands this agent may run"), {
      target: { value: "git status\n npm test \ngit status" },
    });
    fireEvent.click(dialog.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(bot?.allowedCommands).toEqual(["git status", "npm test"]));
  });
});

describe("Claude Code missing", () => {
  test("is only a warning for someone who has an agent that runs on it", async () => {
    const make = async (agent: "claude" | "agy") => {
      const fake = new FakeBotloft();
      fake.system = { ...fake.system, runtimeError: "Claude Code was not found" };
      const crew = fake.addCrew("Ops");
      fake.bot(fake.addBot(crew.id, "Zed").id).agent = agent;
      renderApp(fake);
      await crewOpened("Ops");
    };
    await make("agy");
    expect(screen.queryByText("Agents can't start")).toBeNull();
    cleanup();
    await make("claude");
    expect(await screen.findByText("Agents can't start")).toBeDefined();
  });
});

describe("a Codex agent", () => {
  test("has an effort and a model list under the message box", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const bot = fake.addBot(crew.id, "Cody");
    fake.bot(bot.id).agent = "codex";
    fake.setBotState(bot.id, "idle");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Cody");
    await screen.findByRole("list", { name: "Messages" });
    expect(screen.getByRole("button", { name: /^Effort:/ })).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Model: Default" }));
    expect(await screen.findByRole("menu", { name: "Model" })).toBeDefined();
  });

  test("can compact its conversation, and says Codex does it by itself", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const bot = fake.addBot(crew.id, "Cody");
    fake.bot(bot.id).agent = "codex";
    fake.setBotState(bot.id, "idle");
    fake.setContext(bot.id, {
      usedTokens: 90_000,
      windowTokens: 258_400,
      autoCompactTokens: null,
      compacting: false,
      updatedAt: 0,
    });
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Cody");
    await screen.findByRole("list", { name: "Messages" });
    fireEvent.click(screen.getByRole("button", { name: /Conversation space/ }));
    const panel = within(await screen.findByRole("dialog", { name: "Conversation space" }));
    expect(panel.getByText(/Codex compacts it by itself/)).toBeDefined();
    expect(panel.getByRole("button", { name: "Compact now" })).toBeDefined();
  });
});
