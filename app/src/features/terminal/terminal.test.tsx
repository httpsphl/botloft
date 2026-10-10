import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

async function setup() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "busy");
  const list = fake.chat.tool(scout.id, "PowerShell", {
    summary: "Get-ChildItem",
    explanation: "See what is in the folder",
    input: JSON.stringify({ command: "Get-ChildItem" }),
  });
  fake.chat.finish(list, "report.md\nnotes.md");
  const tests = fake.chat.tool(scout.id, "Bash", {
    summary: "npm test",
    input: JSON.stringify({ command: "npm test" }),
  });
  fake.chat.finish(tests, "1 test failed", true);
  fake.chat.turn(scout.id);
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  // The two calls fold into one line: open it.
  fireEvent.click(screen.getByRole("button", { name: /2 commands/, expanded: false }));
  return { fake, scout };
}

const panel = (name: string) => screen.getByRole("complementary", { name });
const dock = (name: string) =>
  within(screen.getByRole("navigation", { name: "Computer" })).getByRole("button", { name });

describe("the agent's terminal", () => {
  test("a command in the chat opens it, with each command, what it is for and what it printed", async () => {
    const { fake, scout } = await setup();
    fireEvent.click(screen.getByRole("button", { name: "Show in terminal: npm test" }));
    const log = within(panel("Scout's terminal")).getByRole("log");
    expect(await within(log).findByText("# See what is in the folder")).toBeDefined();
    expect(log.textContent).toContain("Get-ChildItem");
    expect(log.textContent).toContain("report.md");
    expect(within(log).getByText("1 test failed").className).toContain("terminal-failed");

    // A new command shows up live, running.
    act(() => {
      fake.chat.tool(scout.id, "Bash", {
        summary: "git status",
        input: JSON.stringify({ command: "git status" }),
      });
    });
    expect(within(log).getByText("git status")).toBeDefined();
    expect(within(log).getByText("running…")).toBeDefined();
  });

  test("the dock moves between the browser, the terminal and the files", async () => {
    await setup();
    fireEvent.click(screen.getByRole("button", { name: "Show in terminal: npm test" }));
    expect(dock("Terminal").getAttribute("aria-current")).toBe("page");

    fireEvent.click(dock("Browser"));
    // It replaces the terminal in place: its content fades up, nothing slides in.
    expect(panel("Scout's browser").className).toContain("panel-swapped");
    fireEvent.click(dock("Files"));
    expect(panel("Files from Scout")).toBeDefined();
    expect(dock("Files").getAttribute("aria-current")).toBe("page");
    fireEvent.click(dock("Terminal"));
    expect(panel("Scout's terminal")).toBeDefined();
  });
});
