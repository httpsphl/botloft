import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

async function openScout() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  fake.chat.tool(scout.id, "mcp__botloft__desktop_look", {
    summary: "To read the list",
    status: "done",
  });
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout };
}

const panel = () => screen.getByRole("complementary", { name: "Scout's desktop" });
const NOTES = { id: 42, title: "Shopping list - Notepad", app: "Notepad" };

describe("the desktop panel", () => {
  test("opens from a desktop tool in the chat and shows the window the bot uses, live", async () => {
    const { fake, scout } = await openScout();
    fireEvent.click(
      screen.getByRole("button", { name: "Watch on the desktop panel: Read a window" }),
    );
    expect(await within(panel()).findByText("Scout has not used your desktop yet")).toBeDefined();
    expect(fake.desktop.watching).toBe(scout.id);

    act(() => {
      fake.desktop.use(scout.id, NOTES, {
        kind: "click",
        target: "Save",
        option: null,
        x: 0.5,
        y: 0.25,
      });
    });
    expect(await within(panel()).findByText("Shopping list - Notepad")).toBeDefined();
    expect(within(panel()).getByText('Clicked "Save"')).toBeDefined();
    expect(within(panel()).getByText("Waiting for the picture…")).toBeDefined();

    act(() => {
      fake.desktop.paint(scout.id, "AAAA");
    });
    const picture = await within(panel()).findByRole("img", { name: "Shopping list - Notepad" });
    expect(picture.getAttribute("src")).toBe("data:image/jpeg;base64,AAAA");
    expect(within(panel()).getByText("Live")).toBeDefined();

    // The bot's cursor lands where it clicked, with a ring.
    const stage = within(panel()).getByRole("figure", { name: "Notepad, as Scout sees it" });
    const cursor = stage.querySelector(".bot-cursor") as HTMLElement;
    expect(cursor.style.left).toBe("50%");
    expect(cursor.style.top).toBe("25%");
    expect(stage.querySelector(".browser-ripple")).not.toBeNull();

    // Closed, nobody watches.
    fireEvent.click(within(panel()).getByRole("button", { name: "Close" }));
    await waitFor(() => expect(fake.desktop.watching).toBeNull());
  });

  test("stops the bot on the desktop and lets it go on", async () => {
    const { fake, scout } = await openScout();
    fireEvent.click(
      screen.getByRole("button", { name: "Watch on the desktop panel: Read a window" }),
    );
    act(() => {
      fake.desktop.use(scout.id, NOTES);
    });
    expect(await within(panel()).findByText("Read the window")).toBeDefined();
    fireEvent.click(within(panel()).getByRole("button", { name: "Stop" }));
    expect(await within(panel()).findByText("You stopped Scout on your desktop")).toBeDefined();
    expect(fake.desktop.state(scout.id).stopped).toBe(true);
    fireEvent.click(within(panel()).getByRole("button", { name: "Let it go on" }));
    await waitFor(() => expect(fake.desktop.state(scout.id).stopped).toBe(false));
    expect(within(panel()).queryByText("You stopped Scout on your desktop")).toBeNull();
  });

  test("opens by itself when the bot starts using the desktop, once per run", async () => {
    const { fake, scout } = await openScout();
    expect(screen.queryByRole("complementary", { name: "Scout's desktop" })).toBeNull();
    act(() => {
      fake.desktop.use(scout.id, NOTES);
    });
    expect(await within(panel()).findByText("Shopping list - Notepad")).toBeDefined();

    // Closed, the next action of the same run leaves it closed.
    fireEvent.click(within(panel()).getByRole("button", { name: "Close" }));
    await waitFor(() =>
      expect(screen.queryByRole("complementary", { name: "Scout's desktop" })).toBeNull(),
    );
    act(() => {
      fake.now += 5_000;
      fake.desktop.use(scout.id, NOTES);
    });
    expect(screen.queryByRole("complementary", { name: "Scout's desktop" })).toBeNull();

    // After a pause, a new run opens it again.
    act(() => {
      fake.now += 3 * 60_000;
      fake.desktop.use(scout.id, NOTES);
    });
    expect(await within(panel()).findByText("Shopping list - Notepad")).toBeDefined();
  });

  test("has the app's options right there, where the owner looks", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.desktop.grant(scout.id, "C:Windows\notepad.exe", "Notepad", "act");
      fake.desktop.use(scout.id, NOTES);
    });
    const options = await within(panel()).findByRole("region", { name: "Options for Notepad" });
    const real = within(options).getByRole("switch", { name: "Real mouse and keyboard: Notepad" });
    fireEvent.click(real);
    fireEvent.click(screen.getByRole("button", { name: "Turn on" }));
    await waitFor(() => expect(fake.desktop.grants[0]?.realInput).toBe(true));
    expect(
      within(options).getByRole("switch", { name: "While you are away: Notepad" }),
    ).toBeDefined();
  });

  test("opens as soon as a desktop tool starts in the chat", async () => {
    const { fake, scout } = await openScout();
    expect(screen.queryByRole("complementary", { name: "Scout's desktop" })).toBeNull();
    act(() => {
      fake.now += 3 * 60_000;
      fake.chat.tool(scout.id, "mcp__botloft__desktop_windows", { status: "running" });
    });
    expect(await within(panel()).findByText("Scout has not used your desktop yet")).toBeDefined();
  });
});
