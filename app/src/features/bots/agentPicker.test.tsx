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
  const [button] = await screen.findAllByRole("button", { name: "New bot" });
  fireEvent.click(button as HTMLElement);
  return within(await screen.findByRole("dialog", { name: "New bot" }));
}

describe("choosing the agent of a new bot", () => {
  test("with only Claude Code there is no choice to make", async () => {
    const dialog = await openNewBot(["claude"]);
    expect(dialog.queryByLabelText("Agent")).toBeNull();
    expect(dialog.getByLabelText("Model")).toBeDefined();
  });

  test("an experimental agent comes with its warning and no model choice", async () => {
    const dialog = await openNewBot(["claude", "agy"]);
    fireEvent.change(dialog.getByLabelText("Agent"), { target: { value: "agy" } });
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
    const [button] = await screen.findAllByRole("button", { name: "New bot" });
    fireEvent.click(button as HTMLElement);
    const dialog = within(await screen.findByRole("dialog", { name: "New bot" }));
    fireEvent.change(dialog.getByLabelText("Agent"), { target: { value: "codex" } });
    expect(dialog.getByRole("note").textContent).toMatch(/cannot change files or run commands yet/);
    expect(dialog.queryByLabelText("Model")).toBeNull();
  });
});
