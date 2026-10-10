import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { ChatItem, ToolItem } from "../../lib/protocol.gen";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { memoryChange, memoryOf, memoryPlaces } from "./memory";

afterEach(cleanup);

const WORKSPACE = "C:\\Users\\owner\\Botloft\\ops\\scout";
const places = memoryPlaces({ workspace: WORKSPACE }, { workFolder: "D:\\Projects\\site" });

function edit(file: string | null, changes: Partial<ToolItem> = {}): ChatItem {
  return {
    id: "cht_1",
    botId: "bot_1",
    createdAt: 0,
    body: {
      kind: "tool",
      toolUseId: "toolu_1",
      name: "Edit",
      summary: "",
      explanation: null,
      input: "{}",
      status: "done",
      output: "",
      file,
      ...changes,
    },
  } as ChatItem;
}

describe("whose memory an edit wrote", () => {
  test("the agent's own, in any case and with either slash", () => {
    expect(memoryOf(edit(`${WORKSPACE}\\CLAUDE.md`), places, WORKSPACE)).toBe("bot");
    expect(memoryOf(edit("c:/users/owner/botloft/ops/scout/claude.md"), places, WORKSPACE)).toBe(
      "bot",
    );
    expect(memoryOf(edit(`${WORKSPACE}\\.claude\\CLAUDE.md`), places, WORKSPACE)).toBe("bot");
    // A relative path is from the bot's folder, where it runs.
    expect(memoryOf(edit("CLAUDE.md"), places, WORKSPACE)).toBe("bot");
  });

  test("the crew's, in the crew folder or the work folder", () => {
    expect(memoryOf(edit("C:\\Users\\owner\\Botloft\\ops\\CLAUDE.md"), places, WORKSPACE)).toBe(
      "crew",
    );
    expect(memoryOf(edit("D:\\Projects\\site\\CLAUDE.md"), places, WORKSPACE)).toBe("crew");
  });

  test("nothing for other files, other agents, or calls that did not finish", () => {
    expect(memoryOf(edit(`${WORKSPACE}\\notes.md`), places, WORKSPACE)).toBeNull();
    expect(
      memoryOf(edit("C:\\Users\\owner\\Botloft\\ops\\writer\\CLAUDE.md"), places, WORKSPACE),
    ).toBeNull();
    expect(memoryOf(edit(null), places, WORKSPACE)).toBeNull();
    for (const status of ["running", "failed"] as const) {
      expect(memoryOf(edit("CLAUDE.md", { status }), places, WORKSPACE)).toBeNull();
    }
  });
});

describe("what changed", () => {
  const tool = () => edit("CLAUDE.md").body as ToolItem;

  test("an edit keeps only the lines that changed", () => {
    const call = tool();
    call.input = JSON.stringify({
      old_string: "# Notes\n- likes tea\n- end",
      new_string: "# Notes\n- likes coffee\n- signs annual\n- end",
    });
    expect(memoryChange(call)).toEqual({
      kind: "edit",
      hunks: [{ removed: ["- likes tea"], added: ["- likes coffee", "- signs annual"] }],
    });
  });

  test("several edits, a whole write, and an input that was cut", () => {
    const call = tool();
    call.input = JSON.stringify({
      edits: [
        { old_string: "a", new_string: "b" },
        { old_string: "same", new_string: "same" },
      ],
    });
    expect(memoryChange(call)).toEqual({ kind: "edit", hunks: [{ removed: ["a"], added: ["b"] }] });
    call.input = JSON.stringify({ file_path: "CLAUDE.md", content: "# All new" });
    expect(memoryChange(call)).toEqual({ kind: "write", text: "# All new" });
    call.input = '{"file_path": "CLAUDE.md", "content": "cut he';
    expect(memoryChange(call)).toBeNull();
  });
});

describe("in the chat", () => {
  test("a memory edit is its own line, out of the group, and opens what changed", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "idle");
    const read = fake.chat.tool(scout.id, "Read", { summary: "brief.md" });
    fake.chat.finish(read, "…");
    const remember = fake.chat.tool(scout.id, "Edit", {
      summary: "CLAUDE.md",
      file: `${scout.workspace}\\CLAUDE.md`,
      input: JSON.stringify({ old_string: "- old", new_string: "- Dana approves" }),
    });
    fake.chat.finish(remember, "ok");
    const run = fake.chat.tool(scout.id, "Bash", { summary: "ls" });
    fake.chat.finish(run, "…");
    fake.chat.turn(scout.id);

    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const line = await screen.findByRole("button", { name: /Updated memory for\s*Scout/ });
    // The calls around it are single lines of their own.
    expect(screen.getByRole("button", { name: /Read a file/ })).toBeDefined();
    expect(screen.queryByRole("button", { name: /Edit a file/ })).toBeNull();

    fireEvent.click(line);
    const changed = screen.getByLabelText("What changed");
    expect(within(changed).getByText("- old")).toBeDefined();
    expect(within(changed).getByText("- Dana approves")).toBeDefined();
  });

  test("the crew's memory names the crew", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "idle");
    const write = fake.chat.tool(scout.id, "Write", {
      file: `${ops.workFolder}\\CLAUDE.md`,
      input: JSON.stringify({ content: "# Ops\nAcme signs annual." }),
    });
    fake.chat.finish(write, "ok");
    fake.chat.turn(scout.id);

    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    fireEvent.click(await screen.findByRole("button", { name: /Updated crew memory for\s*Ops/ }));
    expect(screen.getByText("The memory now says:")).toBeDefined();
    expect(screen.getByText(/Acme signs annual/)).toBeDefined();
  });
});
