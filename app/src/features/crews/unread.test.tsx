import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { prefs, resetPrefs } from "../../shell/prefs";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";

beforeEach(() => {
  // Replies from the start of time count; the app would skip older ones.
  localStorage.setItem("botloft.seenSince", "1");
  localStorage.removeItem("botloft.seen");
});

afterEach(() => {
  cleanup();
  resetPrefs();
  localStorage.removeItem("botloft.unread");
  localStorage.removeItem("botloft.seenSince");
  localStorage.removeItem("botloft.seen");
});

const row = (name: string) =>
  within(sidebar()).getByRole("button", { name: new RegExp(`^${name},`) });

describe("unread replies in the conversation list", () => {
  test("a bot that replied stays marked until the owner opens its chat", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    const writer = fake.addBot(crew.id, "Writer");
    fake.setBotState(scout.id, "idle");
    fake.setBotState(writer.id, "idle");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Writer");
    await screen.findByRole("list", { name: "Messages" });

    act(() => {
      fake.chat.reply(scout.id, "Found three sources.");
      // Something after the reply does not hide it.
      fake.chat.tool(scout.id, "Read");
    });
    expect(row("Scout").getAttribute("aria-label")).toBe("Scout, Idle, unread");

    // The open chat's replies are seen as they come, even ones stamped
    // after the owner opened it.
    act(() => {
      fake.now = Date.now() + 60_000;
      fake.chat.reply(writer.id, "Draft ready.");
    });
    expect(row("Writer").getAttribute("aria-label")).toBe("Writer, Idle");

    openBot("Scout");
    expect(row("Scout").getAttribute("aria-label")).toBe("Scout, Idle");
    expect(row("Writer").getAttribute("aria-label")).toBe("Writer, Idle");
  });

  test("a reply from before the app opened is marked too", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    fake.setBotState(scout.id, "idle");
    fake.chat.reply(scout.id, "Done while you were away.");
    renderApp(fake);
    await crewOpened("Ops");
    expect(row("Scout").getAttribute("aria-label")).toBe("Scout, Idle, unread");
  });

  test("replies from before the first start with unread marks count as seen", async () => {
    localStorage.removeItem("botloft.seenSince");
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    fake.setBotState(scout.id, "idle");
    fake.chat.reply(scout.id, "An old reply.");
    renderApp(fake);
    await crewOpened("Ops");
    expect(row("Scout").getAttribute("aria-label")).toBe("Scout, Idle");
  });

  test("the owner marks a bot unread from its menu, and read again", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    const writer = fake.addBot(crew.id, "Writer");
    fake.setBotState(scout.id, "idle");
    fake.setBotState(writer.id, "idle");
    renderApp(fake);
    await crewOpened("Ops");

    menuItem("Writer", "Mark as unread");
    expect(row("Writer").getAttribute("aria-label")).toBe("Writer, Idle, unread");
    expect(within(sidebar()).getByText("1 unread")).toBeTruthy();

    menuItem("Writer", "Mark as read");
    expect(row("Writer").getAttribute("aria-label")).toBe("Writer, Idle");
    expect(within(sidebar()).queryByText("1 unread")).toBeNull();

    // Marking the open chat unread puts it away, back on the crew's page.
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });
    menuItem("Scout", "Mark as unread");
    await crewOpened("Ops");
    expect(row("Scout").getAttribute("aria-label")).toBe("Scout, Idle, unread");

    // Opening it again reads it.
    openBot("Scout");
    expect(row("Scout").getAttribute("aria-label")).toBe("Scout, Idle");
  });

  test("an unread bot marks the app's icon, unless the owner turned that off", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    fake.setBotState(scout.id, "idle");
    const { host } = renderApp(fake);
    await crewOpened("Ops");
    expect(host.attention).toBe(false);

    act(() => {
      fake.chat.reply(scout.id, "Found three sources.");
    });
    await vi.waitFor(() => expect(host.attention).toBe(true));

    act(() => prefs.markReplies.set(false));
    await vi.waitFor(() => expect(host.attention).toBe(false));
  });
});

/** Picks `label` in the right-click menu of the bot `name`. */
function menuItem(name: string, label: string) {
  fireEvent.contextMenu(row(name), { clientX: 120, clientY: 200 });
  const menu = screen.getByRole("menu", { name: `Actions for ${name}` });
  fireEvent.click(within(menu).getByRole("menuitem", { name: label }));
}
