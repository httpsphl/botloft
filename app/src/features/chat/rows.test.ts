import { describe, expect, test } from "vitest";
import type { ChatBody, ChatItem } from "../../lib/protocol.gen";
import { chatRows, runParts } from "./rows";

const DAY = 24 * 60 * 60 * 1000;
const start = new Date(2026, 8, 28, 9, 0).getTime();
let n = 0;

function item(body: ChatBody, at = start): ChatItem {
  n += 1;
  return { id: `cht_${n}`, botId: "bot_1", body, createdAt: at, updatedAt: at };
}

const reply = (text: string, at?: number) => item({ kind: "reply", text }, at);
const tool = (name: string) =>
  item({
    kind: "tool",
    toolUseId: name,
    name,
    summary: "",
    explanation: null,
    input: "{}",
    status: "done",
    output: null,
    file: null,
  });
const turn = () => item({ kind: "turn", durationMs: 1000, costUsd: null, error: null });
const inbound = (body: string, at?: number) =>
  item(
    {
      kind: "inbound",
      message: {
        id: `msg_${body}`,
        crewId: "crw_1",
        fromKind: "owner",
        fromBotId: null,
        toBotId: "bot_1",
        kind: "note",
        body,
        taskId: null,
        routineId: null,
        attachments: [],
        createdAt: at ?? start,
      },
    },
    at,
  );

describe("chat rows", () => {
  test("a turn runs together until it ends, and messages stand alone", () => {
    const items = [inbound("hi"), reply("one"), tool("Read"), tool("Grep"), turn(), reply("two")];
    const rows = chatRows(items);
    expect(rows.map((row) => row.kind)).toEqual(["day", "inbound", "run", "run"]);
    const first = rows[2];
    expect(first?.kind === "run" && first.items.length).toBe(4);
    if (first?.kind === "run") {
      expect(runParts(first.items).map((part) => part.length)).toEqual([1, 2, 1]);
    }
  });

  test("a new day starts a new run under its date", () => {
    const rows = chatRows([reply("late", start), reply("early", start + DAY)]);
    expect(rows.map((row) => row.kind)).toEqual(["day", "run", "day", "run"]);
  });
});
