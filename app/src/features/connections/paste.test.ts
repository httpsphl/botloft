import { describe, expect, test } from "vitest";
import { parsePasted } from "./paste";

const linkedin = {
  command: "uvx",
  args: ["linkedin-mcp-server"],
  env: { LI_COOKIE: "secret" },
};

describe("pasting a connected tool", () => {
  test("the settings of a project file become one tool each", () => {
    const pasted = parsePasted(
      JSON.stringify({
        mcpServers: { linkedin, web: { type: "http", url: "http://127.0.0.1:8000/mcp" } },
      }),
      "",
    );
    expect(pasted).toEqual({
      ok: true,
      tools: [
        {
          serverId: null,
          name: "linkedin",
          kind: "stdio",
          url: null,
          command: "uvx",
          args: ["linkedin-mcp-server"],
          headers: {},
          env: { LI_COOKIE: "secret" },
          description: "",
        },
        expect.objectContaining({ name: "web", kind: "http", url: "http://127.0.0.1:8000/mcp" }),
      ],
    });
  });

  test("the settings of one server take the name the owner typed", () => {
    expect(parsePasted(JSON.stringify(linkedin), "")).toEqual({ ok: false, reason: "needsName" });
    const pasted = parsePasted(JSON.stringify(linkedin), " LinkedIn ");
    expect(pasted.ok && pasted.tools[0]?.name).toBe("LinkedIn");
  });

  test("a server without a type is a program when it has a command, an address when not", () => {
    const pasted = parsePasted(JSON.stringify({ a: { url: "https://x.test/mcp" } }), "");
    expect(pasted.ok && pasted.tools[0]?.kind).toBe("http");
  });

  test("what is not understood is named, never ignored", () => {
    expect(parsePasted("", "")).toEqual({ ok: false, reason: "empty" });
    expect(parsePasted("{ nope", "")).toEqual({ ok: false, reason: "notJson" });
    expect(parsePasted("[]", "")).toEqual({ ok: false, reason: "notJson" });
    expect(parsePasted('{"mcpServers":{}}', "")).toEqual({ ok: false, reason: "noServers" });
    expect(
      parsePasted(JSON.stringify({ mcpServers: { a: { ...linkedin, timeout: 5 } } }), ""),
    ).toEqual({ ok: false, reason: "unknownField", server: "a", field: "timeout" });
    expect(
      parsePasted(JSON.stringify({ mcpServers: { a: { type: "sse", url: "http://x" } } }), ""),
    ).toEqual({ ok: false, reason: "unsupported", server: "a" });
    expect(
      parsePasted(JSON.stringify({ mcpServers: { a: { command: "x", args: "not a list" } } }), ""),
    ).toEqual({ ok: false, reason: "badValue", server: "a", field: "args" });
    expect(
      parsePasted(JSON.stringify({ mcpServers: { a: { command: "x", env: { N: 1 } } } }), ""),
    ).toEqual({ ok: false, reason: "badValue", server: "a", field: "env" });
    expect(parsePasted(JSON.stringify({ mcpServers: { a: { type: "http" } } }), "")).toEqual({
      ok: false,
      reason: "badValue",
      server: "a",
      field: "url",
    });
  });
});
