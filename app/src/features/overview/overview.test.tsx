import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { tokens } from "../../lib/format";
import type { ContextUsage, OverviewChanges } from "../../lib/protocol.gen";
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

async function openOverview(
  context: ContextUsage | null,
  agent: "claude" | "agy" = "claude",
  changes: OverviewChanges = { repos: [], events: [], prs: [], github: null },
) {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.bot(scout.id).agent = agent;
  fake.setBotState(scout.id, "idle");
  fake.bot(scout.id).context = context;
  fake.changes.byBot.set(scout.id, changes);
  const { host } = renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  fireEvent.click(await screen.findByRole("button", { name: "Show overview" }));
  const panel = await screen.findByRole("complementary", { name: "Overview of Scout" });
  return { fake, scout, panel, host };
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

describe("what runs in the background (spec 33.4)", () => {
  test("says when nothing runs", async () => {
    const { panel } = await openOverview(split(24_200));
    expect(within(panel).getByText("Nothing running in the background.")).toBeTruthy();
  });

  test("lists each task, follows the live list and stops one", async () => {
    const { fake, scout, panel } = await openOverview(split(24_200));
    const seenAt = new Date(2026, 10, 2, 14, 32).getTime();
    fake.setBackground(scout.id, [
      { taskId: "b1", kind: "command", description: "Serve the site preview", seenAt },
      { taskId: "a1", kind: "agent", description: "", seenAt },
    ]);
    const list = await within(panel).findByRole("list", { name: "Running in the background" });
    const rows = within(list).getAllByRole("listitem");
    expect(rows).toHaveLength(2);
    expect(rows[0]?.textContent).toContain("Serve the site preview");
    expect(rows[0]?.textContent).toContain("Command");
    expect(rows[0]?.textContent).toContain("Since");
    // A task without a description goes by its kind.
    expect(rows[1]?.textContent).toContain("Subagent");

    fireEvent.click(within(list).getByRole("button", { name: "Stop: Serve the site preview" }));
    expect(await within(panel).findByRole("button", { name: "Stop: Subagent" })).toBeTruthy();
    expect(within(panel).queryByText("Serve the site preview")).toBeNull();
  });
});

describe("what changed in the agent's git folders (spec 33.5)", () => {
  test("says when none of its folders is in git", async () => {
    const { panel } = await openOverview(split(24_200));
    expect(
      await within(panel).findByText("None of Scout's folders is in a git repository."),
    ).toBeTruthy();
  });

  test("shows each changed file and what the agent did, with the link to its pull request", async () => {
    const { panel, host } = await openOverview(split(24_200), "claude", {
      repos: [
        {
          path: "D:\\Projects\\site",
          name: "site",
          branch: "feat/menu",
          ahead: 2,
          behind: 0,
          files: [
            { path: "src/menu.ts", status: "modified", added: 12, removed: 3 },
            { path: "notes.md", status: "untracked", added: null, removed: null },
          ],
          more: 0,
        },
      ],
      events: [
        {
          kind: "pr",
          sha: null,
          branch: null,
          action: "created",
          number: 14,
          url: "https://github.com/owner/site/pull/14",
          at: 2,
        },
        {
          kind: "commit",
          sha: "4f9a2c1",
          branch: "feat/menu",
          action: "committed",
          number: null,
          url: null,
          at: 1,
        },
      ],
      prs: [
        {
          number: 14,
          url: "https://github.com/owner/site/pull/14",
          title: "Menu page",
          state: "open",
          additions: 160,
          deletions: 6,
          checks: { passed: 3, failed: 1, pending: 0 },
        },
      ],
      github: "ok",
    });
    const files = await within(panel).findByRole("list", { name: "Changed files in site" });
    const rows = within(files).getAllByRole("listitem");
    expect(rows[0]?.textContent).toContain("src/menu.ts");
    expect(
      within(rows[0] as HTMLElement).getByRole("img", { name: "12 lines added, 3 lines removed" }),
    ).toBeTruthy();
    expect(rows[1]?.textContent).toContain("new");
    expect(within(panel).getByText("feat/menu")).toBeTruthy();
    expect(within(panel).getByText("↑2 ↓0")).toBeTruthy();

    const prs = within(panel).getByRole("list", { name: "Pull requests" });
    expect(within(prs).getByText("Menu page")).toBeTruthy();
    expect(within(prs).getByText("Open")).toBeTruthy();
    expect(within(prs).getByText("1 of 4 checks failed")).toBeTruthy();
    const did = within(panel).getByRole("list", { name: "What Scout did in git" });
    expect(within(did).getByText("Pull request #14 opened")).toBeTruthy();
    expect(within(did).getByText("Commit 4f9a2c1 on feat/menu")).toBeTruthy();
    fireEvent.click(within(did).getByRole("button", { name: "Open pull request #14" }));
    await screen.findByRole("complementary", { name: "Overview of Scout" });
    expect(host.opened).toContain("https://github.com/owner/site/pull/14");
  });
});

describe("the pull requests and the connected tools (spec 33.5, 33.6)", () => {
  test("says how to see the checks when gh is missing", async () => {
    const { panel } = await openOverview(split(24_200), "claude", {
      repos: [],
      events: [],
      prs: [],
      github: "missing",
    });
    expect(
      await within(panel).findByText(
        "To see how the checks went, install GitHub CLI (gh) and sign in to it.",
      ),
    ).toBeTruthy();
  });

  test("says when the agent has no connected tools, with the way to Settings > Tools", async () => {
    const { panel } = await openOverview(split(24_200));
    expect(await within(panel).findByText("Scout has no connected tools.")).toBeTruthy();
    fireEvent.click(within(panel).getByRole("button", { name: "Choose in Settings" }));
    const page = await screen.findByRole("region", { name: "Settings" });
    expect(within(page).getByRole("heading", { level: 2, name: "Tools" })).toBeTruthy();
  });
});
