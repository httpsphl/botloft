import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with @scout, its browser open on a sign-in page, watched. */
async function openScout() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "busy");
  fake.browser.open(scout.id, "https://accounts.google.com/", "Sign in");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  fireEvent.click(screen.getByRole("button", { name: /^Show browser/ }));
  await waitFor(() => expect(fake.browser.watching).toBe(scout.id));
  act(() => {
    fake.browser.frame(scout.id, "AAAA");
  });
  return { fake, scout };
}

const panel = () => screen.getByRole("complementary", { name: "Scout's browser" });

describe("the browser in a window of its own", () => {
  test("opens for signing in, says the bot waits, and goes back when the window closes", async () => {
    const { fake, scout } = await openScout();
    // Why is folded until the owner asks.
    expect(within(panel()).queryByText(/like Google/)).toBeNull();
    fireEvent.click(within(panel()).getByRole("button", { name: "How it works" }));
    expect(within(panel()).getByText(/like Google/)).toBeDefined();
    fireEvent.click(within(panel()).getByRole("button", { name: "Sign in in a window" }));

    expect(await within(panel()).findAllByText("Open in a window")).not.toHaveLength(0);
    expect(within(panel()).getByText(/Scout waits meanwhile and keeps the login/)).toBeDefined();
    expect(
      fake.calls.filter((call) => call.method === "browser.window").map((call) => call.params),
    ).toEqual([{ botId: scout.id }]);
    expect(within(panel()).queryByRole("button", { name: "Take control" })).toBeNull();

    act(() => {
      fake.browser.closeWindow(scout.id);
    });
    await waitFor(() => expect(within(panel()).queryByText(/keeps the login/)).toBeNull());
  });

  test("is suggested while the owner is in control, where a sign-in may fail", async () => {
    await openScout();
    fireEvent.click(within(panel()).getByRole("button", { name: "Take control" }));
    expect(await within(panel()).findByText("You are in control")).toBeDefined();
    fireEvent.click(within(panel()).getByRole("button", { name: "How it works" }));
    expect(within(panel()).getByText(/won't let you sign in here\?/)).toBeDefined();
    expect(within(panel()).getByRole("button", { name: "Sign in in a window" })).toBeDefined();
  });

  test("is offered when the bot asks for a hand", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.browser.ask(scout.id, "Sign in to your Google account");
    });
    const asking = await within(panel()).findByText("Sign in to your Google account");
    expect(asking).toBeDefined();
    const hint = /won't let you sign in here\? Use Sign in in a window/;
    expect(within(panel()).getByText(hint)).toBeDefined();
    expect(screen.getByText(/won't let you sign in there, use Sign in in a window/)).toBeDefined();
    fireEvent.click(within(panel()).getByRole("button", { name: "Sign in in a window" }));
    expect(await within(panel()).findByText(/keeps the login/)).toBeDefined();

    // Closing the window is the owner being done.
    act(() => {
      fake.browser.closeWindow(scout.id);
    });
    expect(await screen.findByText("You did it: Sign in to your Google account")).toBeDefined();
  });
});
