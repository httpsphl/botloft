import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { prefs, resetPrefs } from "../shell/prefs";
import { FakeIslandHost } from "./fakeHost";
import { IslandApp } from "./IslandApp";
import { localMidnight } from "./Today";

afterEach(() => {
  cleanup();
  resetPrefs();
});

const host = new FakeIslandHost();

function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  const review = fake.addBot(ops.id, "Review", "Checks");
  return { fake, writer, review };
}

async function openPanel() {
  fireEvent.click(await screen.findByRole("button", { name: /working/ }));
  return screen.findByRole("region", { name: "Botloft at the top of the screen" });
}

describe("the island's agents at work", () => {
  test("show the steps of the turn and the diff of the edit going on", async () => {
    const { fake, writer } = crew();
    fake.setBotState(writer.id, "busy");
    fake.chat.tool(writer.id, "Read", { summary: "notes.md", status: "done" });
    fake.chat.tool(writer.id, "Edit", {
      summary: "report.md",
      status: "running",
      input: JSON.stringify({ old_string: "TODO", new_string: "First\nSecond" }),
      lines: { added: 2, removed: 1 },
    });
    render(<IslandApp api={fake} host={host} />);
    const island = await openPanel();

    expect(within(island).getByText("Writer")).toBeDefined();
    expect(within(island).getByText("Ops · Claude Code")).toBeDefined();
    expect(within(island).getByText("Read a file")).toBeDefined();
    expect(within(island).getByText("Edit a file")).toBeDefined();
    await waitFor(() =>
      expect(
        [...island.querySelectorAll("[data-kind]")].map((row) => row.getAttribute("data-kind")),
      ).toEqual(["removed", "added", "added"]),
    );
    expect(within(island).getByText("report.md")).toBeDefined();

    // The text it writes next takes the place of the diff, as it comes.
    act(() => fake.chat.delta(writer.id, "Done with the "));
    act(() => fake.chat.delta(writer.id, "highlights."));
    expect(await within(island).findByText(/Done with the highlights\./)).toBeDefined();
  });

  test("a second agent at work gets a line of its own", async () => {
    const { fake, writer, review } = crew();
    fake.setBotState(writer.id, "busy");
    fake.setBotState(review.id, "busy");
    fake.chat.tool(review.id, "Bash", { summary: "npm run check-links", status: "running" });
    render(<IslandApp api={fake} host={host} />);
    const island = await openPanel();
    expect(await within(island).findByText("Run a command · npm run check-links")).toBeDefined();
  });

  test("cheer when one finishes, with the first line of its answer", async () => {
    const { fake, writer } = crew();
    fake.setBotState(writer.id, "busy");
    render(<IslandApp api={fake} host={host} />);
    await screen.findByRole("button", { name: "1 working" });

    act(() => {
      fake.chat.reply(writer.id, "**The draft is ready.**\nThree sections.");
      fake.setBotState(writer.id, "idle");
    });
    expect(await screen.findByText("Writer finished", {}, { timeout: 4000 })).toBeDefined();
    expect(screen.getByText(/The draft is ready\./)).toBeDefined();
  });

  test("do not cheer when the owner turned it off", async () => {
    prefs.islandCheer.set(false);
    const { fake, writer } = crew();
    fake.setBotState(writer.id, "busy");
    render(<IslandApp api={fake} host={host} />);
    await screen.findByRole("button", { name: "1 working" });
    act(() => fake.setBotState(writer.id, "idle"));
    await waitFor(() => expect(screen.queryByRole("region")).toBeNull());
    expect(screen.queryByText("Writer finished")).toBeNull();
  });

  test("today counts what the crews did since midnight", async () => {
    const { fake, writer } = crew();
    fake.setBotState(writer.id, "busy");
    fake.chat.tool(writer.id, "Write", {
      status: "done",
      file: "C:\\work\\a.md",
      lines: { added: 12, removed: 0 },
    });
    fake.chat.turn(writer.id);
    render(<IslandApp api={fake} host={host} />);
    const island = await openPanel();
    expect(await within(island).findByText("Today")).toBeDefined();
    expect(within(island).getByText("+12")).toBeDefined();
    expect(within(island).getByText("lines, 1 file")).toBeDefined();
    expect(within(island).getByText("worked the most")).toBeDefined();
  });

  test("the day starts at local midnight", () => {
    const noon = new Date(2026, 9, 25, 12, 30).getTime();
    expect(localMidnight(noon)).toBe(new Date(2026, 9, 25).getTime());
  });
});
