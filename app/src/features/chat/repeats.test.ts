import { describe, expect, test } from "vitest";
import type { ChatItem, ToolItem } from "../../lib/protocol.gen";
import { groupRepeat, REPEAT_FROM, repeatCounts } from "./repeats";

let next = 0;

function call(name: string, input: string, summary = ""): ChatItem {
  next += 1;
  const body: ToolItem & { kind: "tool" } = {
    kind: "tool",
    toolUseId: `toolu_${next}`,
    name,
    summary,
    explanation: null,
    input,
    status: "done",
    output: null,
    file: null,
    lines: null,
  };
  return { id: `cht_${next}`, botId: "bot_1", createdAt: next, body } as ChatItem;
}

function reply(text: string): ChatItem {
  next += 1;
  return {
    id: `cht_${next}`,
    botId: "bot_1",
    createdAt: next,
    body: { kind: "reply", text },
  } as ChatItem;
}

const npmTest = () => call("Bash", '{"command":"npm test"}', "npm test");

describe("the same call made again and again", () => {
  test("counts each call across the whole turn, replies in between", () => {
    const first = [npmTest(), npmTest(), call("Read", '{"file_path":"a.md"}', "a.md")];
    const between = reply("Still failing, trying again.");
    const second = [npmTest(), npmTest(), npmTest()];
    const counts = repeatCounts([...first, between, ...second]);
    expect(second.map((item) => counts.get(item.id))).toEqual([3, 4, 5]);
    expect(counts.get(between.id)).toBeUndefined();
    expect(groupRepeat(first, counts)).toBeNull();
    const repeat = groupRepeat(second, counts);
    expect(repeat?.count).toBe(REPEAT_FROM);
    expect(repeat?.tool.summary).toBe("npm test");
  });

  test("another input is another call", () => {
    const items = [1, 2, 3, 4, 5].map((n) => call("Bash", `{"command":"echo ${n}"}`, `echo ${n}`));
    const counts = repeatCounts(items);
    expect([...counts.values()]).toEqual([1, 1, 1, 1, 1]);
    expect(groupRepeat(items, counts)).toBeNull();
  });

  test("tools made to be called again do not count", () => {
    const looks = Array.from({ length: 8 }, () => call("mcp__botloft__browser_look", "{}"));
    const output = Array.from({ length: 8 }, () => call("BashOutput", '{"bash_id":"1"}'));
    const counts = repeatCounts([...looks, ...output]);
    expect(counts.size).toBe(0);
  });

  test("the group shows its most repeated call", () => {
    const items = [
      ...Array.from({ length: 6 }, () => call("Grep", '{"pattern":"TODO"}', "TODO")),
      ...Array.from({ length: 7 }, () => npmTest()),
    ];
    expect(groupRepeat(items, repeatCounts(items))?.tool.name).toBe("Bash");
    expect(groupRepeat(items, repeatCounts(items))?.count).toBe(7);
  });
});
