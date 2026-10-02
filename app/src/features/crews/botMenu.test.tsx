import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with Scout and Writer, open on the crew's page. */
async function twoBots() {
  const fake = new FakeBotloft();
  const crew = fake.addCrew("Ops");
  const scout = fake.addBot(crew.id, "Scout");
  const writer = fake.addBot(crew.id, "Writer");
  const { host } = renderApp(fake);
  await crewOpened("Ops");
  return { fake, host, crew, scout, writer };
}

const row = (name: string) => within(sidebar()).getByRole("button", { name: new RegExp(name) });

/** Right-clicks the bot's conversation and returns the menu that opens. */
function rightClick(name: string) {
  fireEvent.contextMenu(row(name), { clientX: 120, clientY: 200 });
  return screen.getByRole("menu", { name: `Actions for ${name}` });
}

const labels = (menu: HTMLElement) =>
  within(menu)
    .getAllByRole("menuitem")
    .map((item) => item.textContent);

describe("a bot's right-click menu", () => {
  test("offers what the header's menu offers, without opening the bot", async () => {
    await twoBots();
    const menu = rightClick("Scout");
    const offered = labels(menu);
    expect(offered).toEqual([
      "Edit",
      "Make crew chief",
      "Restart with a new conversation",
      "Open folder",
      "Mark as unread",
      "Archive bot",
      "Delete bot",
    ]);
    // The crew's page is still the one open.
    expect(screen.getByRole("heading", { level: 1, name: "Ops" })).toBeDefined();
    expect(menu.style.left).toBe("120px");
    expect(menu.style.top).toBe("200px");

    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("menu")).toBeNull();
    openBot("Scout");
    fireEvent.click(screen.getByRole("button", { name: "More bot actions" }));
    expect(labels(screen.getByRole("menu"))).toEqual(offered);
  });

  test("acts on the bot that was clicked, not the one that is open", async () => {
    const { fake, host, crew, scout, writer } = await twoBots();
    openBot("Writer");
    await screen.findByRole("heading", { level: 1, name: "Writer" });

    fireEvent.click(within(rightClick("Scout")).getByRole("menuitem", { name: "Open folder" }));
    expect(host.opened).toEqual([scout.workspace]);
    expect(screen.queryByRole("menu")).toBeNull();

    fireEvent.click(within(rightClick("Scout")).getByRole("menuitem", { name: "Make crew chief" }));
    await waitFor(() => expect(fake.crews.get(crew.id)?.leadBotId).toBe(scout.id));
    expect(labels(rightClick("Scout"))).toContain("Stop being chief");
    expect(labels(rightClick("Writer"))).toContain("Make crew chief");

    fireEvent.click(within(rightClick("Scout")).getByRole("menuitem", { name: "Archive bot" }));
    const dialog = screen.getByRole("dialog", { name: "Archive Scout?" });
    expect(sidebar().contains(dialog)).toBe(false);
    fireEvent.click(within(dialog).getByRole("button", { name: "Archive bot" }));
    await waitFor(() =>
      expect(within(sidebar()).queryByRole("button", { name: /Scout/ })).toBeNull(),
    );
    expect(fake.calls.at(-1)).toEqual({ method: "bots.archive", params: { botId: scout.id } });
    // Writer stays open through it all.
    expect(screen.getByRole("heading", { level: 1, name: "Writer" })).toBeDefined();
    expect(fake.bots.get(writer.id)?.archivedAt).toBeNull();
  });

  test("edits from the menu and keeps a paused bot from restarting", async () => {
    const { fake, scout } = await twoBots();
    await fake.call("bots.setPaused", { botId: scout.id, paused: true });
    const menu = rightClick("Scout");
    const fresh = within(menu).getByRole("menuitem", { name: "Restart with a new conversation" });
    expect((fresh as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(within(menu).getByRole("menuitem", { name: "Edit" }));
    expect(screen.getByRole("dialog", { name: "Edit Scout" })).toBeDefined();
  });

  test("opens from the keyboard and walks with the arrows", async () => {
    await twoBots();
    const scout = row("Scout");
    scout.focus();
    // The menu key sends a context menu event without a pointer.
    fireEvent.contextMenu(scout);
    const menu = screen.getByRole("menu", { name: "Actions for Scout" });
    const items = within(menu).getAllByRole("menuitem");
    expect(document.activeElement).toBe(items[0]);
    fireEvent.keyDown(menu, { key: "ArrowDown" });
    expect(document.activeElement).toBe(items[1]);
    fireEvent.keyDown(menu, { key: "ArrowUp" });
    fireEvent.keyDown(menu, { key: "ArrowUp" });
    expect(document.activeElement).toBe(items.at(-1));
    fireEvent.keyDown(menu, { key: "Home" });
    expect(document.activeElement).toBe(items[0]);

    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(scout);
  });

  test("closes on a click elsewhere, and another bot's menu takes its place", async () => {
    await twoBots();
    rightClick("Scout");
    fireEvent.pointerDown(row("Writer"));
    expect(screen.queryByRole("menu")).toBeNull();
    rightClick("Scout");
    fireEvent.pointerDown(row("Writer"));
    expect(rightClick("Writer")).toBeDefined();
    expect(screen.getAllByRole("menu")).toHaveLength(1);
  });
});
