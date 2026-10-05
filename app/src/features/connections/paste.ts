// What the owner pastes to connect a tool (spec 25.2): the `.mcp.json` of a
// project, a bare `{ "mcpServers": { ... } }`, or the settings of a single
// server, as `claude mcp add-json` takes them. Anything this does not know
// is refused with the name of what it did not know, never ignored.

import type { McpSaveParams } from "../../lib/protocol.gen";

export type PasteProblem =
  | { reason: "empty" }
  | { reason: "notJson" }
  | { reason: "noServers" }
  | { reason: "needsName" }
  | { reason: "unsupported"; server: string }
  | { reason: "unknownField"; server: string; field: string }
  | { reason: "badValue"; server: string; field: string };

export type Pasted = { ok: true; tools: McpSaveParams[] } | ({ ok: false } & PasteProblem);

const FIELDS = ["type", "command", "args", "env", "url", "headers"];

const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const isStrings = (value: unknown): value is string[] =>
  Array.isArray(value) && value.every((each) => typeof each === "string");

const isTexts = (value: unknown): value is Record<string, string> =>
  isObject(value) && Object.values(value).every((each) => typeof each === "string");

type Entry = { ok: true; tool: McpSaveParams } | ({ ok: false } & PasteProblem);

function entry(name: string, config: unknown): Entry {
  if (!isObject(config)) {
    return { ok: false, reason: "badValue", server: name, field: name };
  }
  const unknown = Object.keys(config).find((key) => !FIELDS.includes(key));
  if (unknown !== undefined) {
    return { ok: false, reason: "unknownField", server: name, field: unknown };
  }
  const kind = config.type ?? (config.command !== undefined ? "stdio" : "http");
  if (kind !== "stdio" && kind !== "http") {
    return { ok: false, reason: "unsupported", server: name };
  }
  const bad = (field: string): Entry => ({ ok: false, reason: "badValue", server: name, field });
  const args = config.args ?? [];
  const env = config.env ?? {};
  const headers = config.headers ?? {};
  if (!isStrings(args)) {
    return bad("args");
  }
  if (!isTexts(env)) {
    return bad("env");
  }
  if (!isTexts(headers)) {
    return bad("headers");
  }
  if (kind === "stdio" && typeof config.command !== "string") {
    return bad("command");
  }
  if (kind === "http" && typeof config.url !== "string") {
    return bad("url");
  }
  return {
    ok: true,
    tool: {
      serverId: null,
      name,
      kind,
      url: kind === "http" ? (config.url as string) : null,
      command: kind === "stdio" ? (config.command as string) : null,
      args: kind === "stdio" ? args : [],
      headers: kind === "http" ? headers : {},
      env: kind === "stdio" ? env : {},
      description: "",
    },
  };
}

/**
 * Reads what was pasted. A single server's settings have no name of their
 * own: `fallbackName` (what the owner typed) names it.
 */
export function parsePasted(text: string, fallbackName: string): Pasted {
  if (!text.trim()) {
    return { ok: false, reason: "empty" };
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    return { ok: false, reason: "notJson" };
  }
  if (!isObject(parsed)) {
    return { ok: false, reason: "notJson" };
  }
  let named: [string, unknown][];
  if (isObject(parsed.mcpServers)) {
    named = Object.entries(parsed.mcpServers);
  } else if ("command" in parsed || "url" in parsed) {
    if (!fallbackName.trim()) {
      return { ok: false, reason: "needsName" };
    }
    named = [[fallbackName.trim(), parsed]];
  } else {
    named = Object.entries(parsed);
  }
  if (named.length === 0) {
    return { ok: false, reason: "noServers" };
  }
  const tools: McpSaveParams[] = [];
  for (const [name, config] of named) {
    const each = entry(name, config);
    if (!each.ok) {
      return each;
    }
    tools.push(each.tool);
  }
  return { ok: true, tools };
}
