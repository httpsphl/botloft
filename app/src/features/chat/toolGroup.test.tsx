import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { en } from "../../i18n/en";
import { ptBR } from "../../i18n/pt-BR";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { toolSummary } from "./toolSummary";

afterEach(cleanup);

const done = (name: string, failed = false) => ({
  name,
  status: failed ? ("failed" as const) : ("done" as const),
});

describe("a group of tool calls", () => {
  test("sums up in one line, by kind and in order", () => {
    const calls = [
      done("mcp__botloft__browser_open"),
      done("mcp__botloft__browser_look"),
      done("Bash", true),
      done("Read"),
      done("Bash"),
    ];
    expect(toolSummary(calls, en.chat.tools.group)).toBe(
      "Took 2 steps in the browser, ran 2 commands, read 1 file (1 failed)",
    );
    expect(toolSummary(calls, ptBR.chat.tools.group)).toBe(
      "2 passos no navegador, 2 comandos rodados, 1 arquivo lido (1 falha)",
    );
    expect(toolSummary([done("mcp__other__frobnicate")], en.chat.tools.group)).toBe("Used 1 tool");
  });

  test("says what the agent does now, then folds into its summary", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "busy");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const chat = await screen.findByRole("list", { name: "Messages" });

    let read = fake.chat.items[0];
    act(() => {
      read = fake.chat.tool(scout.id, "Read", { summary: "notes.md" });
    });
    // One line for the group, on the call it is on, with no dots under it.
    const line = within(chat).getByRole("button", { name: /Read a file/ });
    expect(line.getAttribute("aria-expanded")).toBe("false");
    expect(within(line).getByLabelText("Running")).toBeDefined();
    expect(within(chat).queryByRole("status", { name: "Working" })).toBeNull();

    act(() => {
      if (read) {
        fake.chat.finish(read, "# Notes");
      }
      fake.chat.tool(scout.id, "Bash", { summary: "npm test", explanation: "Run the tests" });
    });
    expect(within(chat).queryByRole("button", { name: /Read a file/ })).toBeNull();
    const now = within(chat).getByRole("button", { name: /Run a command/ });
    expect(within(now).getByText("Run the tests")).toBeDefined();

    act(() => {
      fake.chat.turn(scout.id);
      fake.setBotState(scout.id, "idle");
    });
    const summary = within(chat).getByRole("button", { name: "Read 1 file, ran 1 command" });
    fireEvent.click(summary);
    expect(summary.getAttribute("aria-expanded")).toBe("true");
    const calls = within(chat).getByRole("list", { name: "Tool calls" });
    expect(within(calls).getAllByRole("listitem")).toHaveLength(2);
    expect(within(calls).getByText("Run the tests")).toBeDefined();
  });

  test("a single finished call stays its own line", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "idle");
    fake.chat.tool(scout.id, "Bash", { summary: "ls", status: "done" });
    fake.chat.turn(scout.id);
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const chat = await screen.findByRole("list", { name: "Messages" });
    const line = within(chat).getByRole("button", { name: /Run a command/ });
    expect(within(line).getByLabelText("Done")).toBeDefined();
  });

  test("a call made again and again gets a mark from the fifth time on", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "idle");
    const npmTest = {
      summary: "npm test",
      input: '{"command":"npm test"}',
      status: "done",
    } as const;
    for (let n = 0; n < 4; n++) {
      fake.chat.tool(scout.id, "Bash", npmTest);
    }
    fake.chat.reply(scout.id, "Still red. Once more.");
    fake.chat.tool(scout.id, "Bash", npmTest);
    fake.chat.turn(scout.id);
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const chat = await screen.findByRole("list", { name: "Messages" });

    // Four in the first group: no mark yet.
    const first = within(chat).getByRole("button", { name: "Ran 4 commands" });
    expect(first.parentElement?.textContent).not.toContain("Same call");
    // The fifth stands alone after the reply and still gets it, with the
    // agent's name and the call on hover.
    const mark = within(chat).getByText("Same call 5×");
    expect(mark.getAttribute("title")).toBe(
      "Scout made the same call 5 times in this turn (Run a command: npm test). It may be going in circles.",
    );
    expect(within(chat).getByRole("button", { name: "Ran 1 command" })).toBeDefined();
  });
});
