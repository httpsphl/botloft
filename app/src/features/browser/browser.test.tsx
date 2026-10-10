import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

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

const toggle = () => screen.getByRole("button", { name: /Show browser|Hide browser/ });
const panel = () => screen.getByRole("complementary", { name: "Scout's browser" });

describe("browser panel", () => {
  test("watches the agent's browser while open, and says when it has not opened one", async () => {
    const { fake, scout } = crew();
    await openScout(fake);
    fireEvent.click(toggle());
    expect(await within(panel()).findByText("Scout hasn't opened the browser yet")).toBeDefined();
    await waitFor(() => expect(fake.browser.watching).toBe(scout.id));

    fireEvent.click(within(panel()).getByRole("button", { name: "Close" }));
    await waitFor(() => expect(fake.browser.watching).toBeNull());
    expect(fake.browser.watches).toEqual([scout.id, null]);
  });

  test("shows the live page, its address, and the agent's cursor where it clicks", async () => {
    const { fake, scout } = crew();
    fake.browser.open(scout.id, "https://example.com/signin", "Sign in");
    await openScout(fake);
    fireEvent.click(toggle());
    const address = await within(panel()).findByRole("textbox", { name: "Address" });
    expect((address as HTMLInputElement).value).toBe("https://example.com/signin");
    await waitFor(() => expect(fake.browser.watching).toBe(scout.id));

    act(() => {
      fake.browser.frame(scout.id, "AAAA");
      fake.browser.act(scout.id, "click", { x: 640, y: 400, label: "Sign in" });
    });
    const screenshot = within(panel()).getByRole("figure", { name: "What Scout sees" });
    const image = screenshot.querySelector("img");
    expect(image?.getAttribute("src")).toBe("data:image/jpeg;base64,AAAA");
    const cursor = screenshot.querySelector(".bot-cursor") as HTMLElement;
    expect(cursor.style.left).toBe("50%");
    expect(cursor.style.top).toBe("50%");
    expect(within(panel()).getByText("Live")).toBeDefined();
    expect(await within(panel()).findByText("Clicked Sign in")).toBeDefined();

    // Another bot's actions do not move this cursor.
    const other = fake.addBot(scout.crewId, "Writer");
    act(() => {
      fake.browser.act(other.id, "open", { label: "elsewhere.com" });
    });
    expect(within(panel()).queryByText("Opened elsewhere.com")).toBeNull();
  });

  test("gives the page the shape of the room on the desk, and shows it as large as fits", async () => {
    const { fake, scout } = crew();
    fake.browser.open(scout.id, "https://example.com/", "Example");
    // The room on the desk, inside its edge, as the app measures it: all
    // of it is the page's, so no band of the desk shows beside it.
    vi.spyOn(Element.prototype, "clientWidth", "get").mockReturnValue(560);
    vi.spyOn(Element.prototype, "clientHeight", "get").mockReturnValue(804);
    await openScout(fake);
    fireEvent.click(toggle());
    const resized = () =>
      fake.calls.filter((call) => call.method === "browser.resize").map((call) => call.params);
    await waitFor(() =>
      expect(resized()).toEqual([{ botId: scout.id, width: 560, height: 804, scale: 100 }]),
    );
    // A narrow room keeps the desktop layout's width, in the room's shape.
    expect(fake.browser.size(scout.id)).toEqual({ width: 800, height: 1148 });

    act(() => {
      fake.browser.frame(scout.id, "AAAA");
    });
    // The bot's cursor lands where it acted on the taller page.
    const screenshot = within(panel()).getByRole("figure", { name: "What Scout sees" });
    act(() => {
      fake.browser.act(scout.id, "click", { x: 400, y: 1148, label: "More" });
    });
    const cursor = screenshot.querySelector(".bot-cursor") as HTMLElement;
    expect(cursor.style.top).toBe("100%");

    // Nobody watches anymore: the page is back to its own size.
    fireEvent.click(within(panel()).getByRole("button", { name: "Close" }));
    await waitFor(() => expect(fake.browser.size(scout.id)).toEqual({ width: 1280, height: 800 }));
  });

  test("says when the browser rests, and it is live again once someone uses it", async () => {
    const { fake, scout } = crew();
    fake.browser.open(scout.id, "https://example.com/", "Example");
    await openScout(fake);
    fireEvent.click(toggle());
    await waitFor(() => expect(fake.browser.watching).toBe(scout.id));
    act(() => {
      fake.browser.frame(scout.id, "AAAA");
    });
    expect(within(panel()).getByText("Live")).toBeDefined();

    // The bot's turn ended: the panel closes by itself, and the button does
    // not say the bot is using it.
    act(() => {
      fake.browser.rest(scout.id);
    });
    await waitFor(() => expect(toggle().getAttribute("aria-label")).toBe("Show browser"));
    // Opened again, the page is still there and the panel says it rests.
    fireEvent.click(toggle());
    const resting = within(panel()).getByText("Resting");
    expect(resting.title).toContain("Scout isn't using the browser");
    expect(within(panel()).queryByText("Live")).toBeNull();
    expect(within(panel()).getByRole("figure").querySelector("img")).not.toBeNull();

    // The owner's hands wake it.
    fireEvent.click(await within(panel()).findByRole("button", { name: "Take control" }));
    expect(await within(panel()).findByText("You are in control")).toBeDefined();
    expect(within(panel()).queryByText("Resting")).toBeNull();
    expect(fake.browser.state(scout.id).resting).toBe(false);
  });

  test("closes when the browser rests and opens again when the agent wakes it, but not out of the owner's hands", async () => {
    const { fake, scout } = crew();
    await openScout(fake);
    act(() => {
      fake.browser.open(scout.id, "https://example.com/", "Example");
    });
    await waitFor(() => expect(toggle().getAttribute("aria-label")).toBe("Hide browser"));

    act(() => {
      fake.browser.rest(scout.id);
    });
    await waitFor(() =>
      expect(screen.queryByRole("complementary", { name: "Scout's browser" })).toBeNull(),
    );

    // The bot's next step wakes it, and the panel follows.
    act(() => {
      fake.browser.open(scout.id, "https://example.com/next", "Next");
    });
    await waitFor(() => expect(toggle().getAttribute("aria-label")).toBe("Hide browser"));

    // With the browser in the owner's hands, it stays.
    await waitFor(() => expect(fake.browser.watching).toBe(scout.id));
    act(() => {
      fake.browser.frame(scout.id, "AAAA");
    });
    fireEvent.click(await within(panel()).findByRole("button", { name: "Take control" }));
    expect(await within(panel()).findByText("You are in control")).toBeDefined();
    act(() => {
      fake.browser.rest(scout.id);
    });
    expect(toggle().getAttribute("aria-label")).toBe("Hide browser");
  });

  test("opens by itself when the agent starts browsing, and the button marks it once closed", async () => {
    const { fake, scout } = crew();
    await openScout(fake);
    fireEvent.click(screen.getByRole("button", { name: "Show files" }));
    expect(toggle().getAttribute("aria-label")).toBe("Show browser");
    act(() => {
      fake.browser.open(scout.id, "https://example.com/", "Example");
    });
    // It takes the place of the files, so the owner sees the bot browse.
    expect(await screen.findByRole("complementary", { name: "Scout's browser" })).toBeDefined();
    expect(screen.queryByRole("complementary", { name: "Files from Scout" })).toBeNull();

    fireEvent.click(toggle());
    expect(toggle().getAttribute("aria-label")).toBe("Show browser: Scout is using the browser");
    // Going on browsing does not open it again.
    act(() => {
      fake.browser.open(scout.id, "https://example.com/more", "More");
    });
    expect(screen.queryByRole("complementary", { name: "Scout's browser" })).toBeNull();
  });

  test("a browser already open when the agent is opened stays behind its button", async () => {
    const { fake, scout } = crew();
    fake.browser.open(scout.id, "https://example.com/", "Example");
    await openScout(fake);
    expect(toggle().getAttribute("aria-label")).toBe("Show browser: Scout is using the browser");
    expect(screen.queryByRole("complementary", { name: "Scout's browser" })).toBeNull();
  });

  test("a closed browser keeps its last page, faded; one that failed says why", async () => {
    const { fake, scout } = crew();
    fake.browser.open(scout.id, "https://example.com/", "Example");
    await openScout(fake);
    fireEvent.click(toggle());
    await waitFor(() => expect(fake.browser.watching).toBe(scout.id));
    act(() => {
      fake.browser.frame(scout.id, "BBBB");
    });
    act(() => {
      fake.browser.close(scout.id);
    });
    expect(await within(panel()).findByText("Browser closed")).toBeDefined();
    expect(within(panel()).getByRole("figure").querySelector("img")?.className).toContain(
      "grayscale",
    );

    act(() => {
      fake.browser.set(scout.id, {
        status: "failed",
        error: "Microsoft Edge was not found on this computer",
      });
    });
    expect(await within(panel()).findByText("The browser couldn't open")).toBeDefined();
    expect(
      within(panel()).getByText("Microsoft Edge was not found on this computer"),
    ).toBeDefined();
  });

  test("a browser line in the chat opens the panel", async () => {
    const { fake, scout } = crew();
    fake.chat.tool(scout.id, "mcp__botloft__browser_open", {
      summary: "https://example.com",
      status: "done",
    });
    fake.chat.tool(scout.id, "Bash", { summary: "ls", status: "done" });
    await openScout(fake);
    fireEvent.click(
      screen.getByRole("button", { name: "Took 1 step in the browser, ran 1 command" }),
    );
    expect(screen.getAllByRole("button", { name: /Watch in browser/ })).toHaveLength(1);
    fireEvent.click(screen.getByRole("button", { name: "Watch in browser: Open a page" }));
    expect(await screen.findByRole("complementary", { name: "Scout's browser" })).toBeDefined();
  });
});

describe("site requests", () => {
  test("show the site and its address, and allowing it answers the request", async () => {
    const { fake, scout } = crew();
    const item = fake.chat.ask(
      scout.id,
      "mcp__botloft__browser",
      "wikipedia.org",
      JSON.stringify({ site: "wikipedia.org", url: "https://wikipedia.org/wiki/Bread" }),
    );
    await openScout(fake);
    const card = await screen.findByRole("region", { name: "Scout asks to use wikipedia.org" });
    expect(within(card).getByText("https://wikipedia.org/wiki/Bread")).toBeDefined();
    fireEvent.click(within(card).getByRole("button", { name: "Allow" }));
    await waitFor(() =>
      expect(fake.calls).toContainEqual({
        method: "approvals.answer",
        params: {
          approvalId: item.body.kind === "approval" ? item.body.approvalId : "",
          allow: true,
        },
      }),
    );
  });
});
