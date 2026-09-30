import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";

beforeEach(() => localStorage.clear());
afterEach(cleanup);

/** A crew "Ops" with @scout and @writer, idle. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes the notes");
  fake.setBotState(scout.id, "idle");
  fake.setBotState(writer.id, "idle");
  return { fake, scout, writer };
}

async function open(name: string) {
  openBot(name);
  await screen.findByRole("region", { name: `Chat with ${name}` });
}

async function started(fake: FakeBotloft) {
  renderApp(fake);
  await crewOpened("Ops");
  await open("Scout");
}

const panel = (name: string) => screen.queryByRole("complementary", { name });
const button = (name: RegExp) => screen.getByRole("button", { name });

describe("the panel beside a bot's chat", () => {
  test("comes back with the bot, each bot with its own", async () => {
    const { fake, scout } = crew();
    await started(fake);
    fireEvent.click(button(/^Show browser/));
    await waitFor(() => expect(fake.browser.watching).toBe(scout.id));

    await open("Writer");
    expect(panel("Scout's browser")).toBeNull();
    expect(panel("Writer's browser")).toBeNull();
    fireEvent.click(button(/^Show files/));
    expect(panel("Files from Writer")).not.toBeNull();

    await open("Scout");
    const back = panel("Scout's browser");
    expect(back).not.toBeNull();
    // It is there at once: only a panel opened by hand or by the bot slides.
    expect(back?.className).not.toContain("panel-opening");
    await waitFor(() => expect(fake.browser.watching).toBe(scout.id));
    expect(panel("Files from Scout")).toBeNull();

    await open("Writer");
    expect(panel("Files from Writer")).not.toBeNull();
    // Another one opened now slides, from the width of the one it replaces.
    fireEvent.click(button(/^Show details/));
    expect(panel("About Writer")?.className).toContain("panel-opening");
  });

  test("closed by the owner, it stays closed when they come back", async () => {
    const { fake, scout } = crew();
    await started(fake);
    // The bot starts browsing: its browser opens by itself.
    act(() => {
      fake.browser.open(scout.id, "https://example.com/", "Example");
    });
    await waitFor(() => expect(panel("Scout's browser")).not.toBeNull());
    fireEvent.click(button(/^Hide browser/));

    await open("Writer");
    await open("Scout");
    expect(panel("Scout's browser")).toBeNull();
    expect(button(/^Show browser/).getAttribute("aria-label")).toBe(
      "Show browser: Scout is using the browser",
    );
  });

  test("one that opened by itself comes back too, and the crew's page in between changes nothing", async () => {
    const { fake, scout } = crew();
    await started(fake);
    act(() => {
      fake.screens.draft(scout.id, "C:\\Work\\site\\home.html", "<h1>Home");
    });
    await waitFor(() => expect(panel("Scout's screens")).not.toBeNull());

    fireEvent.click(within(sidebar()).getByRole("button", { name: "Ops" }));
    await crewOpened("Ops");
    expect(panel("Scout's screens")).toBeNull();
    await open("Scout");
    expect(panel("Scout's screens")).not.toBeNull();
  });

  test("what the bot did while the owner was away stays behind its button, beside the panel that came back", async () => {
    const { fake, scout } = crew();
    await started(fake);
    fireEvent.click(button(/^Show files/));
    await open("Writer");
    act(() => {
      fake.browser.open(scout.id, "https://example.com/", "Example");
    });

    await open("Scout");
    expect(panel("Files from Scout")).not.toBeNull();
    expect(panel("Scout's browser")).toBeNull();
    expect(button(/^Show browser/).getAttribute("aria-label")).toBe(
      "Show browser: Scout is using the browser",
    );
    // What it starts from now on takes the place of that panel, as always.
    act(() => {
      fake.screens.draft(scout.id, "C:\\Work\\site\\home.html", "<h1>Home");
    });
    await waitFor(() => expect(panel("Scout's screens")).not.toBeNull());
    expect(panel("Files from Scout")).toBeNull();
  });
});
