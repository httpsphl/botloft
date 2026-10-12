import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, renderApp } from "../../test/app";

afterEach(cleanup);

async function openNewBot(agents: ("claude" | "agy")[]) {
  const fake = new FakeBotloft();
  fake.system = { ...fake.system, enabledAgents: agents };
  fake.addCrew("Ops");
  renderApp(fake);
  await crewOpened("Ops");
  const [button] = await screen.findAllByRole("button", { name: "New agent" });
  fireEvent.click(button as HTMLElement);
  return within(await screen.findByRole("dialog", { name: "New agent" }));
}

describe("choosing the agent of a new agent", () => {
  test("with only Claude Code there is no choice to make", async () => {
    const dialog = await openNewBot(["claude"]);
    expect(dialog.queryByLabelText("Runs on")).toBeNull();
    expect(dialog.getByLabelText("Model")).toBeDefined();
  });

  test("an experimental agent comes with its warning and no model choice", async () => {
    const dialog = await openNewBot(["claude", "agy"]);
    fireEvent.change(dialog.getByLabelText("Runs on"), { target: { value: "agy" } });
    expect(dialog.getByRole("note").textContent).toMatch(/cannot ask you/);
    expect(dialog.queryByLabelText("Model")).toBeNull();
  });
});

describe("Codex in the new-bot dialog", () => {
  test("says what it cannot do yet", async () => {
    const fake = new FakeBotloft();
    fake.system = { ...fake.system, enabledAgents: ["claude", "codex"] };
    fake.addCrew("Ops");
    renderApp(fake);
    await crewOpened("Ops");
    const [button] = await screen.findAllByRole("button", { name: "New agent" });
    fireEvent.click(button as HTMLElement);
    const dialog = within(await screen.findByRole("dialog", { name: "New agent" }));
    fireEvent.change(dialog.getByLabelText("Runs on"), { target: { value: "codex" } });
    expect(dialog.getByRole("note").textContent).toMatch(
      /asks you before it changes a file or runs a command/,
    );
    expect(dialog.queryByLabelText("Model")).toBeNull();
  });
});

describe("an agent whose program is not on the computer", () => {
  test("its chat says why it keeps restarting", async () => {
    const fake = new FakeBotloft();
    fake.system = {
      ...fake.system,
      enabledAgents: ["claude", "codex"],
      agentChecks: [{ agent: "codex", version: null, path: null }],
    };
    const crew = fake.addCrew("Ops");
    const bot = fake.addBot(crew.id, "Cody");
    fake.bot(bot.id).agent = "codex";
    fake.setBotState(bot.id, "backoff");
    renderApp(fake);
    await crewOpened("Ops");
    fireEvent.click(await screen.findByRole("button", { name: /^Cody,/ }));
    expect(await screen.findByText("Codex was not found")).toBeDefined();
    expect(screen.getByText(/tell Botloft where it is in Settings/)).toBeDefined();
  });
});
