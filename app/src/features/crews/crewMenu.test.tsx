import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";

afterEach(cleanup);

async function twoCrews() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const site = fake.addCrew("Site");
  fake.addBot(ops.id, "Scout");
  fake.addBot(site.id, "Writer");
  const { host } = renderApp(fake);
  await crewOpened("Ops");
  return { fake, host, ops, site };
}

/** Right-clicks the crew in the list and returns the menu that opens. */
function rightClick(name: string) {
  fireEvent.contextMenu(within(sidebar()).getByRole("button", { name: new RegExp(`^${name}`) }), {
    clientX: 120,
    clientY: 200,
  });
  return screen.getByRole("menu", { name: `Actions for ${name}` });
}

const item = (menu: HTMLElement, name: string) => within(menu).getByRole("menuitem", { name });

describe("a crew's right-click menu", () => {
  test("offers what the crew page offers, without opening the crew", async () => {
    await twoCrews();
    openBot("Scout");
    const menu = rightClick("Site");
    expect(
      within(menu)
        .getAllByRole("menuitem")
        .map((one) => one.textContent),
    ).toEqual([
      "New bot",
      "Pause crew",
      "Rename",
      "Open work folder",
      "Change work folder…",
      "Archive crew",
      "Delete crew",
    ]);
    // Scout's chat stays open.
    expect(screen.getByRole("list", { name: "Messages" })).toBeDefined();
  });

  test("pauses, opens the folder, and opens the dialogs over the list", async () => {
    const { fake, host, site } = await twoCrews();
    fireEvent.click(item(rightClick("Site"), "Pause crew"));
    await waitFor(() =>
      expect(fake.calls.at(-1)).toMatchObject({
        method: "crews.setPaused",
        params: { crewId: site.id, paused: true },
      }),
    );
    expect(within(rightClick("Site")).getByRole("menuitem", { name: "Resume crew" })).toBeDefined();
    fireEvent.keyDown(document.activeElement ?? document.body, { key: "Escape" });

    fireEvent.click(item(rightClick("Site"), "Open work folder"));
    await waitFor(() => expect(host.opened).toEqual([site.workFolder]));

    fireEvent.click(item(rightClick("Site"), "Rename"));
    expect(await screen.findByRole("dialog", { name: /Rename/ })).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

    fireEvent.click(item(rightClick("Site"), "Delete crew"));
    expect(await screen.findByRole("dialog", { name: /Delete Site/ })).toBeDefined();
  });
});
