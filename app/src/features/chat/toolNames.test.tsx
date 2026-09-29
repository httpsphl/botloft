import { act, cleanup, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { en } from "../../i18n/en";
import { ptBR } from "../../i18n/pt-BR";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { toolAction, toolDetail, toolTitle } from "./toolNames";

afterEach(cleanup);

const call = (name: string, summary: string, input: unknown) => ({
  name,
  summary,
  input: JSON.stringify(input),
});

describe("tool names", () => {
  test("read as plain actions in the owner's language", () => {
    expect(toolTitle("mcp__botloft__send_message", en.tools)).toBe("Send a message");
    expect(toolTitle("mcp__botloft__browser_open", ptBR.tools)).toBe("Abrir uma página");
    expect(toolTitle("Bash", ptBR.tools)).toBe("Rodar um comando");
    expect(toolAction("Write", ptBR.tools)).toBe("escrever um arquivo");
    // One with no name here keeps its own.
    expect(toolTitle("mcp__other__frobnicate", en.tools)).toBe("frobnicate");
    expect(toolAction("mcp__other__frobnicate", ptBR.tools)).toBe("usar frobnicate");
    expect(toolTitle("constructor", en.tools)).toBe("constructor");
  });

  test("say what a call is about without English words, from its input", () => {
    const message = call("mcp__botloft__send_message", "to @writer", { to: "writer", body: "hi" });
    expect(toolDetail(message, ptBR.tools)).toBe("@writer");
    const load = call("ToolSearch", "load send_message, Read", {
      query: "select:mcp__botloft__send_message,Read",
    });
    expect(toolDetail(load, ptBR.tools)).toBe("Mandar uma mensagem, Ler um arquivo");
    expect(toolDetail(call("TodoWrite", "Updated the plan", { todos: [] }), en.tools)).toBe("");
    expect(toolDetail(call("mcp__botloft__complete_task", "task tsk_1", {}), en.tools)).toBe("");
    expect(toolDetail(call("mcp__botloft__browser_click", "e4", { ref: "e4" }), en.tools)).toBe("");
    const scroll = call("mcp__botloft__browser_scroll", "down", { to: "down" });
    expect(toolDetail(scroll, ptBR.tools)).toBe("para baixo");
    expect(toolDetail(call("Bash", "Run the tests", { command: "npm test" }), en.tools)).toBe(
      "Run the tests",
    );
  });

  test("the conversation list names the tool and the request", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "busy");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const row = () => screen.getByRole("button", { name: /^Scout,/ });
    act(() => {
      fake.chat.tool(scout.id, "mcp__botloft__send_message", {
        summary: "@writer",
        input: '{"to":"writer","body":"hi"}',
      });
    });
    expect(within(row()).getByText("Send a message · @writer")).toBeDefined();
    act(() => {
      fake.chat.ask(scout.id, "Bash", "rm -rf build", '{"command":"rm -rf build"}');
    });
    expect(within(row()).getByText("Waiting for approval: run a command")).toBeDefined();
  });
});
