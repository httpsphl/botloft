import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { prefs, resetPrefs } from "../../shell/prefs";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { reducedMotion } from "../../ui/motion";

afterEach(() => {
  cleanup();
  resetPrefs();
});

async function scout() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const bot = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(bot.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, bot };
}

async function openSettings(page: string) {
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const dialog = screen.getByRole("dialog", { name: "Settings" });
  fireEvent.click(within(dialog).getByRole("tab", { name: page }));
  return dialog;
}

const sends = (fake: FakeBotloft) => fake.calls.filter((call) => call.method === "messages.send");

describe("the app's own choices", () => {
  test("Ctrl+Enter sends and Enter starts a line, when chosen", async () => {
    const { fake } = await scout();
    const dialog = await openSettings("Chat");
    fireEvent.click(within(dialog).getByRole("switch", { name: "Enter sends the message" }));
    expect(dialog.textContent).toContain("Enter starts a new line, and Ctrl+Enter sends.");
    fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));

    const field = screen.getByLabelText("Message to Scout");
    fireEvent.change(field, { target: { value: "two\nlines" } });
    expect(screen.getByText("Ctrl+Enter to send, Enter for a new line")).toBeDefined();
    fireEvent.keyDown(field, { key: "Enter" });
    expect(sends(fake)).toHaveLength(0);
    fireEvent.keyDown(field, { key: "Enter", ctrlKey: true });
    const chat = screen.getByRole("list", { name: "Messages" });
    expect(await within(chat).findByText(/two\s+lines/)).toBeDefined();
    expect(sends(fake)).toHaveLength(1);
  });

  test("the browser and the screens stay behind their buttons, when chosen", async () => {
    prefs.followBot.set(false);
    const { fake, bot } = await scout();
    act(() => {
      fake.browser.open(bot.id, "https://example.com/", "Example");
      fake.screens.draft(bot.id, "C:\\Work\\home.html", "<h1>Home");
    });
    expect(screen.queryByRole("complementary", { name: "Scout's browser" })).toBeNull();
    expect(
      screen.getByRole("button", { name: "Show browser: Scout is using the browser" }),
    ).toBeDefined();
    expect(screen.queryByRole("complementary", { name: /screens/i })).toBeNull();
  });

  test("less motion stills the window", async () => {
    renderApp(new FakeBotloft());
    const dialog = await openSettings("Appearance");
    expect(reducedMotion()).toBe(false);
    fireEvent.click(within(dialog).getByRole("switch", { name: "Less motion" }));
    expect(document.documentElement.dataset.motion).toBe("less");
    expect(reducedMotion()).toBe(true);
    fireEvent.click(within(dialog).getByRole("switch", { name: "Less motion" }));
    expect(document.documentElement.dataset.motion).toBeUndefined();
  });
});
