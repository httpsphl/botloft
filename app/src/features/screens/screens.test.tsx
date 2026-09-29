import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

beforeEach(() => localStorage.clear());
afterEach(cleanup);

/** A crew "Ops" with @scout, idle. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Designs the site");
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

const toggle = () => screen.getByRole("button", { name: /Show screens|Hide screens/ });
const panel = () => screen.getByRole("complementary", { name: "Scout's screens" });
const frames = (within_: HTMLElement) =>
  [...within_.querySelectorAll("iframe")].map((frame) => frame.getAttribute("src"));

describe("design area", () => {
  test("says when the bot has made no screens", async () => {
    const { fake } = crew();
    await openScout(fake);
    fireEvent.click(toggle());
    expect(await within(panel()).findByText("Scout hasn't made any screens yet")).toBeDefined();
  });

  test("shows each screen as an artboard, and one on its own with its device", async () => {
    const { fake, scout } = crew();
    const home = fake.screens.add(scout.id, "home.html", "<h1>Home</h1>", { at: fake.now - 1000 });
    fake.screens.add(scout.id, "app.html", "<h1>App</h1>", { device: "mobile" });
    const { host } = await openScout(fake);
    fireEvent.click(toggle());
    const board = await within(panel()).findByRole("list", { name: "Screens" });
    const artboards = within(board).getAllByRole("listitem");
    expect(artboards).toHaveLength(2);
    expect(artboards[0]?.textContent).toContain("app.html");
    expect(artboards[0]?.textContent).toContain("Phone");
    expect(artboards[1]?.textContent).toContain("Computer");
    expect(frames(board)).toContain(home.url);
    for (const frame of board.querySelectorAll("iframe")) {
      expect(frame.getAttribute("sandbox")).toBe(
        "allow-scripts allow-forms allow-popups allow-modals",
      );
    }

    fireEvent.click(within(board).getByRole("button", { name: "Open home.html" }));
    const focus = await within(panel()).findByRole("button", { name: "All screens" });
    expect(focus).toBeDefined();
    fireEvent.click(within(panel()).getByRole("button", { name: "Tablet" }));
    expect(
      within(panel()).getByRole("button", { name: "Tablet" }).getAttribute("aria-pressed"),
    ).toBe("true");
    expect(localStorage.getItem("botloft.screens.devices")).toContain("tablet");
    fireEvent.click(within(panel()).getByRole("button", { name: "Open" }));
    await waitFor(() => expect(host.openedFiles).toEqual([home.path]));

    fireEvent.click(within(panel()).getByRole("button", { name: "All screens" }));
    const again = await within(panel()).findByRole("list", { name: "Screens" });
    expect(within(again).getAllByRole("listitem")[1]?.textContent).toContain("Tablet");
  });

  test("a screen being written opens the area and grows version by version", async () => {
    const { fake, scout } = crew();
    await openScout(fake);
    expect(screen.queryByRole("complementary", { name: "Scout's screens" })).toBeNull();
    const path = "C:\\Work\\site\\landing.html";

    act(() => {
      fake.screens.draft(scout.id, path, "<h1>Bak");
    });
    const board = await within(panel()).findByRole("list", { name: "Screens" });
    const artboard = within(board).getByRole("listitem");
    expect(artboard.textContent).toContain("landing.html");
    expect(artboard.textContent).toContain("Writing…");
    const first = frames(artboard);
    expect(first[0]).toContain(encodeURIComponent("<h1>Bak"));

    // The next version loads behind the one on show.
    act(() => {
      fake.screens.draft(scout.id, path, "<h1>Bakery</h1><p>Fresh");
    });
    await waitFor(() => expect(frames(artboard)).toHaveLength(2));
    expect(frames(artboard)[1]).toContain(encodeURIComponent("Fresh"));

    // Done: the file on disk is the screen now.
    fake.screens.add(scout.id, "landing.html", "<h1>Bakery</h1>", { folder: "site" });
    act(() => {
      fake.screens.draft(scout.id, path, "<h1>Bakery</h1>", true);
    });
    await waitFor(() => expect(within(panel()).queryByText("Writing…")).toBeNull());
    expect(within(panel()).getByText("landing.html")).toBeDefined();
  });

  test("a screen being written takes the place of another panel, once", async () => {
    const { fake, scout } = crew();
    await openScout(fake);
    fireEvent.click(screen.getByRole("button", { name: "Show files" }));
    act(() => {
      fake.screens.draft(scout.id, "C:\\Work\\a.html", "<p>");
    });
    expect(await within(panel()).findByRole("list", { name: "Screens" })).toBeDefined();
    expect(screen.queryByRole("complementary", { name: "Files from Scout" })).toBeNull();

    // Closed, it stays closed while that screen is written; the button marks it.
    fireEvent.click(toggle());
    act(() => {
      fake.screens.draft(scout.id, "C:\\Work\\a.html", "<p>More");
    });
    await waitFor(() =>
      expect(toggle().getAttribute("aria-label")).toBe("Show screens: Scout is drawing a screen"),
    );
    expect(screen.queryByRole("complementary", { name: "Scout's screens" })).toBeNull();
  });

  test("the browser in the owner's hands stays when a screen starts", async () => {
    const { fake, scout } = crew();
    fake.browser.open(scout.id, "https://example.com/", "Example");
    await openScout(fake);
    fireEvent.click(screen.getByRole("button", { name: /^Show browser/ }));
    await waitFor(() => expect(fake.browser.watching).toBe(scout.id));
    act(() => {
      fake.browser.frame(scout.id, "AAAA");
    });
    fireEvent.click(await screen.findByRole("button", { name: "Take control" }));
    expect(await screen.findByText("You are in control")).toBeDefined();
    act(() => {
      fake.screens.draft(scout.id, "C:\\Work\\a.html", "<p>");
    });
    await waitFor(() =>
      expect(toggle().getAttribute("aria-label")).toBe("Show screens: Scout is drawing a screen"),
    );
    expect(screen.getByRole("complementary", { name: "Scout's browser" })).toBeDefined();
  });

  test("a Write of an HTML file in the chat opens its screen", async () => {
    const { fake, scout } = crew();
    const page = fake.screens.add(scout.id, "about.html", "<h1>About</h1>");
    fake.chat.tool(scout.id, "Write", { summary: "about.html", file: page.path, status: "done" });
    fake.chat.tool(scout.id, "Write", {
      summary: "notes.md",
      file: "C:\\Work\\notes.md",
      status: "done",
    });
    await openScout(fake);
    expect(screen.getAllByRole("button", { name: /Show in screens/ })).toHaveLength(1);
    fireEvent.click(screen.getByRole("button", { name: "Show in screens: about.html" }));
    expect(await within(panel()).findByRole("button", { name: "All screens" })).toBeDefined();
    expect(within(panel()).getByText("about.html")).toBeDefined();
  });
});
