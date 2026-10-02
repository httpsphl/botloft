import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { BotFile } from "../../lib/protocol.gen";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { runParts } from "./rows";

afterEach(cleanup);

/** A crew "Ops" with @scout, idle. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  return { fake, scout };
}

/** Scout shares `files` with `share_file`, as the daemon answers it. */
function share(fake: FakeBotloft, botId: string, files: BotFile[], failed = false) {
  const call = fake.chat.tool(botId, "mcp__botloft__share_file", {
    summary: files.map((file) => file.name).join(", "),
    input: JSON.stringify({ files: files.map((file) => file.path) }),
  });
  const answer = failed
    ? "There is no file at C:\\Work\\gone.pdf"
    : JSON.stringify({ shown: files });
  return fake.chat.finish(call, answer, failed);
}

async function openScout(fake: FakeBotloft) {
  const rendered = renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return rendered;
}

const cards = () => screen.findByRole("list", { name: "Files shared by Scout" });

describe("shared files", () => {
  test("show as cards that open, save a copy and preview", async () => {
    const { fake, scout } = crew();
    const report = fake.files.add(scout.id, "report.pdf", { text: "%PDF-1.7", folder: "final" });
    const notes = fake.files.add(scout.id, "notes.md", { text: "# Notes" });
    share(fake, scout.id, [report, notes]);
    const { host } = await openScout(fake);

    const list = await cards();
    const items = within(list).getAllByRole("listitem");
    expect(items).toHaveLength(2);
    expect(items[0]?.textContent).toContain("report.pdf");
    expect(items[0]?.textContent).toContain("PDF · 8 B");
    // A card, not a tool line.
    expect(screen.queryByRole("button", { name: /Share a file/ })).toBeNull();

    const first = within(items[0] as HTMLElement);
    fireEvent.click(first.getByRole("button", { name: "Save as…" }));
    fireEvent.click(first.getByRole("button", { name: "Open" }));
    await waitFor(() => expect(host.savedFiles).toEqual([report.path]));
    expect(host.openedFiles).toEqual([report.path]);

    fireEvent.click(within(items[1] as HTMLElement).getByRole("button", { name: /^Preview/ }));
    const panel = await screen.findByRole("complementary", { name: "Files from Scout" });
    expect(await within(panel).findByRole("heading", { name: "Notes" })).toBeDefined();
  });

  test("a file the panel's list leaves out still previews from the card", async () => {
    const { fake, scout } = crew();
    // The list skips files older than the bot; the card still knows it.
    const old = fake.files.add(scout.id, "old.md", { text: "# Older than the bot", listed: false });
    share(fake, scout.id, [old]);
    await openScout(fake);

    const list = await cards();
    fireEvent.click(within(list).getByRole("button", { name: /^Preview/ }));
    const panel = await screen.findByRole("complementary", { name: "Files from Scout" });
    expect(await within(panel).findByRole("heading", { name: "Older than the bot" })).toBeDefined();
  });

  test("a share that failed stays a tool line with its error", async () => {
    const { fake, scout } = crew();
    const gone = fake.files.add(scout.id, "gone.pdf");
    share(fake, scout.id, [gone], true);
    await openScout(fake);
    expect(await screen.findByRole("button", { name: /Share a file/ })).toBeDefined();
    expect(screen.queryByRole("list", { name: "Files shared by Scout" })).toBeNull();
  });

  test("shared files stand apart from the tool lines around them", () => {
    const { fake, scout } = crew();
    const file = fake.files.add(scout.id, "a.csv", { text: "a" });
    const read = fake.chat.finish(fake.chat.tool(scout.id, "Read"), "ok");
    const shared = share(fake, scout.id, [file]);
    const grep = fake.chat.finish(fake.chat.tool(scout.id, "Grep"), "ok");
    const write = fake.chat.finish(fake.chat.tool(scout.id, "Write"), "ok");
    expect(runParts([read, shared, grep, write]).map((part) => part.length)).toEqual([1, 1, 2]);
  });
});
