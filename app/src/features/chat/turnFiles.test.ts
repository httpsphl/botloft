import { describe, expect, test } from "vitest";
import type { BotFile, ChatItem } from "../../lib/protocol.gen";
import { madeInTurn, sharedIn } from "./turnFiles";

const END = 1_000_000;

function file(name: string, modifiedAt: number, mediaType = "application/octet-stream"): BotFile {
  return {
    path: `C:\\Work\\${name}`,
    name,
    folder: "",
    mediaType,
    size: 10,
    modifiedAt,
    writtenByBot: false,
  };
}

describe("madeInTurn", () => {
  const turn = { at: END, durationMs: 60_000 };

  test("keeps the media changed during the turn, oldest first", () => {
    const files = [
      file("late.mp4", END - 1000, "video/mp4"),
      file("early.png", END - 59_000, "image/png"),
      file("voice.m4a", END - 30_000),
      file("notes.md", END - 1000, "text/markdown"),
      file("before.png", END - 90_000, "image/png"),
      file("after.png", END + 10_000, "image/png"),
    ];
    expect(madeInTurn(files, turn, []).map((one) => one.name)).toEqual([
      "early.png",
      "voice.m4a",
      "late.mp4",
    ]);
  });

  test("leaves out what the turn shared, however its path is written", () => {
    const files = [file("cover.png", END - 1000, "image/png"), file("b.png", END, "image/png")];
    expect(madeInTurn(files, turn, ["c:/work/COVER.png"]).map((one) => one.name)).toEqual([
      "b.png",
    ]);
  });
});

describe("sharedIn", () => {
  test("lists the files a run shared", () => {
    const shown = [file("a.png", 1, "image/png")];
    const item = (name: string, output: string, status: "done" | "failed"): ChatItem =>
      ({
        id: `cht_${name}`,
        botId: "bot_1",
        createdAt: 1,
        updatedAt: 1,
        body: {
          kind: "tool",
          toolUseId: name,
          name,
          summary: "",
          explanation: null,
          input: "{}",
          status,
          output,
          file: null,
          lines: null,
        },
      }) as ChatItem;
    expect(
      sharedIn([
        item("mcp__botloft__share_file", JSON.stringify({ shown }), "done"),
        item("mcp__botloft__share_file", JSON.stringify({ shown }), "failed"),
        item("Bash", JSON.stringify({ shown }), "done"),
      ]),
    ).toEqual(["C:\\Work\\a.png"]);
  });
});
