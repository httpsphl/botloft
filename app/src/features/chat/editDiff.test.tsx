import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { type DiffLine, diffLines, editDiffs, splitLines } from "./lineDiff";

afterEach(cleanup);

/** Lines only in the new text and only in the old, as the daemon counts. */
function counts(lines: DiffLine[]) {
  return {
    added: lines.filter((line) => line.kind === "added").length,
    removed: lines.filter((line) => line.kind === "removed").length,
  };
}

describe("the lines an edit changes", () => {
  test("split the way the daemon splits them", () => {
    expect(splitLines("")).toEqual([]);
    expect(splitLines("a\nb\n")).toEqual(["a", "b"]);
    expect(splitLines("a\r\n\r\nb")).toEqual(["a", "", "b"]);
    expect(splitLines("\n")).toEqual([""]);
  });

  test("keep what both share and change the rest, removed first", () => {
    const diff = diffLines(
      "fn a() {\n    one();\n    two();\n}",
      "fn a() {\n    one();\n    three();\n    four();\n}",
    );
    expect(diff.map((line) => `${line.kind[0]} ${line.text}`)).toEqual([
      "s fn a() {",
      "s     one();",
      "r     two();",
      "a     three();",
      "a     four();",
      "s }",
    ]);
    // The same counts as the daemon's tests (crates/botloft-core/src/edits.rs).
    expect(counts(diff)).toEqual({ added: 2, removed: 1 });
    expect(counts(diffLines("a\nb\nc\nd", "b\na\nc\nd\ne"))).toEqual({ added: 2, removed: 1 });
    expect(counts(diffLines("let x = 1;", "let x = 2;"))).toEqual({ added: 1, removed: 1 });
    expect(counts(diffLines("a\r\nb\r\n", ""))).toEqual({ added: 0, removed: 2 });
  });

  test("come from the input of each edit tool, when it came whole", () => {
    const edit = JSON.stringify({ file_path: "a.md", old_string: "a", new_string: "a\nb" });
    expect(editDiffs("Edit", edit)?.map(counts)).toEqual([{ added: 1, removed: 0 }]);
    const multi = JSON.stringify({
      edits: [
        { old_string: "a", new_string: "b" },
        { old_string: "x\ny", new_string: "x", replace_all: true },
      ],
    });
    expect(editDiffs("MultiEdit", multi)?.map(counts)).toEqual([
      { added: 1, removed: 1 },
      { added: 0, removed: 1 },
    ]);
    const write = JSON.stringify({ file_path: "new.txt", content: "one\ntwo\n" });
    expect(editDiffs("Write", write)?.map(counts)).toEqual([{ added: 2, removed: 0 }]);
    // Cut to size, only the path (Codex, Antigravity), or another tool.
    expect(editDiffs("Edit", `${edit.slice(0, 20)}…`)).toBeNull();
    expect(editDiffs("Edit", JSON.stringify({ file_path: "a.md" }))).toBeNull();
    expect(editDiffs("Read", JSON.stringify({ content: "a" }))).toBeNull();
  });
});

describe("an edit in the chat", () => {
  async function chat() {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const list = await screen.findByRole("list", { name: "Messages" });
    return { fake, scout, list };
  }

  test("shows its counts on the line and the diff when opened", async () => {
    const { fake, scout, list } = await chat();
    act(() => {
      fake.chat.tool(scout.id, "Edit", {
        summary: "notes.md",
        status: "done",
        input: JSON.stringify({
          file_path: "C:\\Work\\notes.md",
          old_string: "# Notes\nTODO",
          new_string: "# Notes\nFirst\nSecond",
        }),
        lines: { added: 2, removed: 1 },
      });
    });
    const line = within(list).getByRole("button", { name: /Edit a file/ });
    expect(within(line).getByRole("img", { name: "2 lines added, 1 line removed" })).toBeDefined();
    fireEvent.click(line);
    expect(within(list).getByText("Changes")).toBeDefined();
    expect(within(list).queryByText("Input")).toBeNull();
    const kinds = [...list.querySelectorAll("[data-kind]")].map((row) => [
      row.getAttribute("data-kind"),
      row.textContent?.slice(1),
    ]);
    expect(kinds).toEqual([
      ["same", "# Notes"],
      ["removed", "TODO"],
      ["added", "First"],
      ["added", "Second"],
    ]);
  });

  test("says when the change is too long to show, and shows no counts when it failed", async () => {
    const { fake, scout, list } = await chat();
    act(() => {
      fake.chat.tool(scout.id, "Write", {
        summary: "big.md",
        status: "done",
        input: '{"file_path":"big.md","content":"one\\ntwo…',
        lines: { added: 900, removed: 0 },
      });
      // A reply in between keeps the two calls on lines of their own.
      fake.chat.reply(scout.id, "Now the old notes.");
      fake.chat.tool(scout.id, "Edit", {
        summary: "gone.md",
        status: "failed",
        input: JSON.stringify({ file_path: "gone.md", old_string: "a", new_string: "b" }),
        lines: { added: 1, removed: 1 },
      });
    });
    const [write, failed] = within(list).getAllByRole("button", { name: /a file/ });
    if (!write || !failed) {
      throw new Error("two lines");
    }
    expect(within(write).getByRole("img", { name: "900 lines added" })).toBeDefined();
    fireEvent.click(write);
    expect(within(list).getByText("This change is too long to show here.")).toBeDefined();
    expect(within(list).getByText("Input")).toBeDefined();
    expect(within(failed).queryByRole("img")).toBeNull();
  });
});
