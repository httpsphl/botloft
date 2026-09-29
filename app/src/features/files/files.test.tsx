import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with @scout, idle. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  return { fake, scout };
}

async function openScout(fake: FakeBotloft) {
  const rendered = renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return rendered;
}

const toggle = () => screen.getByRole("button", { name: /Show files|Hide files/ });
const panel = () => screen.getByRole("complementary", { name: "Files from Scout" });

describe("files panel", () => {
  test("shows what the bot made, newest first, and previews text", async () => {
    const { fake, scout } = crew();
    const now = fake.now;
    fake.files.add(scout.id, "sources.md", { text: "# Sources\n\n- one", at: now - 60_000 });
    fake.files.add(scout.id, "report.pdf", { text: "%PDF", folder: "final", at: now - 1000 });
    await openScout(fake);

    fireEvent.click(toggle());
    const list = await within(panel()).findByRole("list", { name: "Files" });
    const names = within(list)
      .getAllByRole("button")
      .map((row) => row.textContent);
    expect(names[0]).toContain("report.pdf");
    expect(names[0]).toContain("final");
    expect(names[1]).toContain("sources.md");

    fireEvent.click(within(list).getByRole("button", { name: /sources\.md/ }));
    expect(await within(panel()).findByRole("heading", { name: "Sources" })).toBeDefined();
    fireEvent.click(within(panel()).getByRole("button", { name: "All files" }));
    expect(within(panel()).getByRole("list", { name: "Files" })).toBeDefined();
  });

  test("Open and Show in folder go to the shell, for the file's own path", async () => {
    const { fake, scout } = crew();
    const file = fake.files.add(scout.id, "plan.csv", { text: "a,b" });
    const { host } = await openScout(fake);
    fireEvent.click(toggle());
    fireEvent.click(await within(panel()).findByRole("button", { name: /plan\.csv/ }));
    fireEvent.click(await within(panel()).findByRole("button", { name: "Open" }));
    fireEvent.click(within(panel()).getByRole("button", { name: "Show in folder" }));
    await waitFor(() => expect(host.openedFiles).toEqual([file.path]));
    expect(host.revealed).toEqual([file.path]);
  });

  test("a file with no preview says so instead of reading it", async () => {
    const { fake, scout } = crew();
    fake.files.add(scout.id, "deck.docx", { text: "x" });
    await openScout(fake);
    fireEvent.click(toggle());
    fireEvent.click(await within(panel()).findByRole("button", { name: /deck\.docx/ }));
    expect(await within(panel()).findByText(/no preview for this kind of file/)).toBeDefined();
    expect(fake.calls.some((call) => call.method === "files.read")).toBe(false);
  });

  test("an empty bot says it has made nothing yet", async () => {
    const { fake } = crew();
    await openScout(fake);
    fireEvent.click(toggle());
    expect(await within(panel()).findByText("Scout hasn't made any files yet")).toBeDefined();
  });

  test("a file that shows up while the panel is closed puts a count on the button", async () => {
    const { fake, scout } = crew();
    await openScout(fake);
    expect(toggle().getAttribute("aria-label")).toBe("Show files");

    // The bot works, makes a file, and stops: the list is read once more.
    act(() => {
      fake.setBotState(scout.id, "busy");
    });
    await new Promise((resolve) => setTimeout(resolve, 5));
    act(() => {
      fake.files.add(scout.id, "result.txt", { text: "done", at: Date.now() });
      fake.setBotState(scout.id, "idle");
    });
    await waitFor(() => expect(toggle().getAttribute("aria-label")).toBe("Show files: 1 new file"));

    fireEvent.click(toggle());
    expect(await within(panel()).findByText("New")).toBeDefined();
    fireEvent.click(toggle());
    expect(toggle().getAttribute("aria-label")).toBe("Show files");
  });
});
