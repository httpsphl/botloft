import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

const SEARCH = "https://www.example.com/search?q=rye";
const MILL = "https://mill.example/flour/rye";

/** A crew "Ops" with @scout, its browser on a page with another tab behind it. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "busy");
  fake.browser.open(scout.id, SEARCH, "Rye · Example");
  fake.browser.openTab(scout.id, MILL, "Rye flour · The Mill");
  return { fake, scout };
}

async function openPanel(fake: FakeBotloft, botId: string) {
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  fireEvent.click(screen.getByRole("button", { name: /^Show browser/ }));
  await waitFor(() => expect(fake.browser.watching).toBe(botId));
  act(() => {
    fake.browser.frame(botId, "AAAA");
  });
}

const panel = () => screen.getByRole("complementary", { name: "Scout's browser" });
const tabs = () => within(within(panel()).getByRole("tablist", { name: "Tabs" }));
const tab = (name: string) => tabs().getByRole("tab", { name });
const selected = () =>
  tabs()
    .getAllByRole("tab")
    .map((one) => `${one.getAttribute("aria-label")}${one.ariaSelected === "true" ? " *" : ""}`);
const address = () => within(panel()).getByRole("textbox", { name: "Address" }) as HTMLInputElement;
const called = (fake: FakeBotloft, method: string) =>
  fake.calls.filter((call) => call.method === method).map((call) => call.params);

describe("the tabs of the bot's browser", () => {
  test("show what the bot has open, by title and site, and only show until the owner takes it", async () => {
    const { fake, scout } = crew();
    await openPanel(fake, scout.id);
    expect(selected()).toEqual([
      "Rye · Example, example.com",
      "Rye flour · The Mill, mill.example *",
    ]);
    expect(address().value).toBe(MILL);
    expect(address().readOnly).toBe(true);

    // A page opens another tab: it shows up, and is the active one.
    act(() => {
      fake.browser.openTab(scout.id, "https://papers.example/rye.pdf", "");
    });
    expect(selected()).toEqual([
      "Rye · Example, example.com",
      "Rye flour · The Mill, mill.example",
      "papers.example *",
    ]);

    // The browser is the bot's: clicking moves nothing.
    fireEvent.click(tab("Rye · Example, example.com"));
    const add = within(panel()).getByRole("button", { name: "New tab" });
    expect(add.getAttribute("aria-disabled")).toBe("true");
    expect(add.title).toBe("Take control to switch tabs or open a new one");
    fireEvent.click(add);
    expect(called(fake, "browser.switchTab")).toEqual([]);
    expect(called(fake, "browser.newTab")).toEqual([]);
  });

  test("the page reloads at any time, without taking the browser", async () => {
    const { fake, scout } = crew();
    await openPanel(fake, scout.id);
    fireEvent.click(within(panel()).getByRole("button", { name: "Reload" }));
    await waitFor(() => expect(fake.browser.reloads).toEqual([scout.id]));
    expect(called(fake, "browser.take")).toEqual([]);
    expect(fake.browser.state(scout.id).control).toBe("bot");
  });

  test("in the owner's hands they switch, a new one opens, and the address is theirs to type", async () => {
    const { fake, scout } = crew();
    await openPanel(fake, scout.id);
    fireEvent.click(within(panel()).getByRole("button", { name: "Take control" }));
    expect(await within(panel()).findByText("You are in control")).toBeDefined();
    expect(within(panel()).getByText("Scout goes on in the tab you leave open.")).toBeDefined();

    fireEvent.click(tab("Rye · Example, example.com"));
    await waitFor(() =>
      expect(selected()).toEqual([
        "Rye · Example, example.com *",
        "Rye flour · The Mill, mill.example",
      ]),
    );
    expect(address().value).toBe(SEARCH);

    // A new tab is blank, and the owner says where it goes.
    fireEvent.click(within(panel()).getByRole("button", { name: "New tab" }));
    await waitFor(() => expect(selected()[2]).toBe("New tab *"));
    await waitFor(() => expect(document.activeElement).toBe(address()));
    expect(address().value).toBe("");
    fireEvent.change(address(), { target: { value: "bakery.example/orders" } });
    fireEvent.keyDown(address(), { key: "Enter" });
    await waitFor(() => expect(address().value).toBe("https://bakery.example/orders"));
    expect(called(fake, "browser.open")).toEqual([
      { botId: scout.id, url: "bakery.example/orders" },
    ]);
    expect(selected()[2]).toBe("bakery.example *");

    // Escape drops what was typed; what is no address says so.
    fireEvent.change(address(), { target: { value: "somewhere" } });
    fireEvent.keyDown(address(), { key: "Escape" });
    expect(address().value).toBe("https://bakery.example/orders");
    fireEvent.change(address(), { target: { value: "two words" } });
    fireEvent.keyDown(address(), { key: "Enter" });
    expect(await screen.findByText(/^Could not open that address/)).toBeDefined();

    // Back with the bot, the tabs only show again.
    fireEvent.click(within(panel()).getByRole("button", { name: "Done, give it back to Scout" }));
    await waitFor(() => expect(address().readOnly).toBe(true));
    fireEvent.click(tab("Rye · Example, example.com"));
    expect(called(fake, "browser.switchTab")).toHaveLength(1);
  });
});
