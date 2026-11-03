import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { tokens } from "../../lib/format";
import type { ContextUsage } from "../../lib/protocol.gen";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

/** As Claude Code 2.1.309 split a conversation (spec 33.3). */
const split = (usedTokens: number): ContextUsage => ({
  usedTokens,
  windowTokens: 1_000_000,
  autoCompactTokens: 967_000,
  compacting: false,
  updatedAt: 1,
  parts: [
    { kind: "messages", tokens: 4_442 },
    { kind: "instructions", tokens: 3_341 },
    { kind: "tools", tokens: 13_908 },
    { kind: "memory", tokens: 299 },
    { kind: "skills", tokens: 2_210 },
    { kind: "buffer", tokens: 33_000 },
  ],
  memoryFiles: [{ path: "C:\\Users\\owner\\Botloft\\ops\\scout\\CLAUDE.md", tokens: 299 }],
});

async function openOverview(context: ContextUsage | null, agent: "claude" | "agy" = "claude") {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.bot(scout.id).agent = agent;
  fake.setBotState(scout.id, "idle");
  fake.bot(scout.id).context = context;
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  fireEvent.click(await screen.findByRole("button", { name: "Show overview" }));
  const panel = await screen.findByRole("complementary", { name: "Overview of Scout" });
  return { fake, scout, panel };
}

/** Each part's name and tokens, as the list shows them. */
function parts(panel: HTMLElement): string[] {
  const list = within(panel).getByRole("list", { name: "What fills it" });
  return within(list)
    .getAllByRole("listitem")
    .map((item) => item.textContent ?? "");
}

describe("the overview's conversation space (spec 33.3)", () => {
  test("shows the total, the parts and what is free", async () => {
    const { panel } = await openOverview(split(24_200));
    expect(within(panel).getByText("2.4% used")).toBeTruthy();
    expect(within(panel).getByText(`${tokens(24_200)} of ${tokens(1_000_000)}`)).toBeTruthy();
    expect(within(panel).getByRole("img", { name: "Conversation space, 2.4% used" })).toBeTruthy();
    const rows = parts(panel);
    expect(rows.map((row) => row.split(/\d/)[0])).toEqual([
      "Conversation",
      "Claude Code's instructions",
      "Claude Code's tools",
      "Memory",
      "Skills",
      "Room kept to compact",
      "Free",
    ]);
    // Free is the window less what is used and the room kept to compact.
    expect(rows.at(-1)).toContain(tokens(1_000_000 - 24_200 - 33_000));
  });

  test("the conversation grows with the total between Claude Code's answers", async () => {
    const { fake, scout, panel } = await openOverview(split(24_200));
    // A model request moved the total; the split waits for the next answer.
    fake.setContext(scout.id, { ...split(24_200), usedTokens: 54_200 });
    const conversation = await within(panel).findByText((_, node) =>
      node?.tagName === "LI" && (node.textContent ?? "").startsWith("Conversation")
        ? (node.textContent ?? "").includes(tokens(54_200 - 19_758))
        : false,
    );
    expect(conversation).toBeTruthy();
  });

  test("lists the memory files by name and folder", async () => {
    const { panel } = await openOverview(split(24_200));
    const files = within(panel).getByRole("list", { name: "Memory files" });
    const item = within(files).getByRole("listitem");
    expect(item.textContent).toContain("CLAUDE.md");
    expect(item.textContent).toContain("scout");
    expect(item.getAttribute("title")).toBe("C:\\Users\\owner\\Botloft\\ops\\scout\\CLAUDE.md");
  });

  test("an engine that gives only the total says so", async () => {
    const { panel } = await openOverview({ ...split(120_000), parts: [], memoryFiles: [] }, "agy");
    expect(within(panel).getByText("12% used")).toBeTruthy();
    expect(
      within(panel).getByText("This engine only gives the total, not what fills it."),
    ).toBeTruthy();
    expect(within(panel).queryByRole("button", { name: "Compact now" })).toBeNull();
  });

  test("before the first answer it says the size is not known yet", async () => {
    const { panel } = await openOverview(null);
    expect(
      within(panel).getByText("Scout hasn't said how full its conversation is yet."),
    ).toBeTruthy();
  });
});
