import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with @scout, its browser open on a sign-in page. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "busy");
  fake.browser.open(scout.id, "https://github.com/login", "Sign in to GitHub");
  return { fake, scout };
}

async function openScout(fake: FakeBotloft) {
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
}

const panel = () => screen.getByRole("complementary", { name: "Scout's browser" });
const called = (fake: FakeBotloft, method: string) =>
  fake.calls.filter((call) => call.method === method).map((call) => call.params);

async function openPanel(fake: FakeBotloft, botId: string) {
  fireEvent.click(screen.getByRole("button", { name: /^Show browser/ }));
  await waitFor(() => expect(fake.browser.watching).toBe(botId));
  act(() => {
    fake.browser.frame(botId, "AAAA");
  });
}

describe("the owner's hands in the browser", () => {
  test("taking it sends clicks, keys and pasted text to the page, and giving it back ends it", async () => {
    const { fake, scout } = crew();
    await openScout(fake);
    await openPanel(fake, scout.id);
    // The pill under the screen says who has it.
    const pill = () =>
      within(panel())
        .getAllByRole("status")
        .find((s) => s.closest(".control-pill"));
    expect(pill()?.textContent).toContain("Scout is in control");
    fireEvent.click(within(panel()).getByRole("button", { name: "Take control" }));
    expect(await within(panel()).findByText("You are in control")).toBeDefined();
    expect(pill()?.textContent).toContain("Done, give it back to Scout");
    expect(called(fake, "browser.take")).toEqual([{ botId: scout.id }]);

    const page = within(panel()).getByRole("application", {
      name: "Scout's browser, in your hands",
    });
    vi.spyOn(page, "getBoundingClientRect").mockReturnValue({
      left: 10,
      top: 20,
      width: 640,
      height: 400,
    } as DOMRect);
    fireEvent.mouseDown(page, { clientX: 330, clientY: 220, button: 0, buttons: 1, detail: 1 });
    fireEvent.mouseUp(page, { clientX: 330, clientY: 220, button: 0, buttons: 0, detail: 1 });
    const keys = within(panel()).getByRole("textbox", { name: "What you type goes to the page" });
    fireEvent.keyDown(keys, { key: "a", code: "KeyA" });
    fireEvent.keyDown(keys, { key: "Shift", code: "ShiftLeft", shiftKey: true });
    fireEvent.keyDown(keys, { key: "a", code: "KeyA", ctrlKey: true });
    // Ctrl+V pastes the owner's clipboard as text.
    fireEvent.keyDown(keys, { key: "v", code: "KeyV", ctrlKey: true });
    fireEvent.paste(keys, { clipboardData: { getData: () => "hunter2" } });
    fireEvent.keyDown(keys, { key: "Enter", code: "Enter" });

    await waitFor(() => expect(fake.browser.inputs).toHaveLength(6));
    expect(fake.browser.inputs).toEqual([
      {
        kind: "mouse",
        action: "down",
        x: 640,
        y: 400,
        button: "left",
        buttons: 1,
        clicks: 1,
        modifiers: 0,
      },
      {
        kind: "mouse",
        action: "up",
        x: 640,
        y: 400,
        button: "left",
        buttons: 0,
        clicks: 1,
        modifiers: 0,
      },
      { kind: "key", key: "a", code: "KeyA", modifiers: 0 },
      { kind: "key", key: "a", code: "KeyA", modifiers: 2 },
      { kind: "text", text: "hunter2" },
      { kind: "key", key: "Enter", code: "Enter", modifiers: 0 },
    ]);

    fireEvent.click(within(panel()).getByRole("button", { name: "Done, give it back to Scout" }));
    expect(await within(panel()).findByRole("button", { name: "Take control" })).toBeDefined();
    expect(called(fake, "browser.release")).toEqual([{ botId: scout.id }]);
    expect(within(panel()).queryByRole("application")).toBeNull();
  });

  test("a request for a hand shows in the chat and the panel, and the card opens the browser in the owner's hands", async () => {
    const { fake, scout } = crew();
    fake.browser.ask(scout.id, "Sign in to your GitHub account");
    await openScout(fake);
    // The bot needs a hand: its browser opens by itself.
    expect(await screen.findByRole("complementary", { name: "Scout's browser" })).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Hide browser" }));
    expect(
      screen.getByRole("button", { name: "Show browser: Scout needs you in the browser" }),
    ).toBeDefined();
    const card = await screen.findByRole("region", {
      name: "Scout asks: Sign in to your GitHub account",
    });
    expect(within(card).getByText("github.com")).toBeDefined();

    fireEvent.click(within(card).getByRole("button", { name: "Take the browser" }));
    expect(await within(panel()).findByText("You are in control")).toBeDefined();
    expect(called(fake, "browser.take")).toEqual([{ botId: scout.id }]);
    expect(within(panel()).getByText("Scout asks: Sign in to your GitHub account")).toBeDefined();
    act(() => {
      fake.browser.frame(scout.id, "AAAA");
    });
    expect(within(panel()).getByRole("application")).toBeDefined();

    // Giving it back says the owner is done.
    fireEvent.click(within(panel()).getByRole("button", { name: "Done, give it back to Scout" }));
    expect(await screen.findByText("You did it: Sign in to your GitHub account")).toBeDefined();
    expect(within(panel()).getByRole("button", { name: "Take control" })).toBeDefined();
  });

  test("the panel shows the request, and the owner may say no in the chat", async () => {
    const { fake, scout } = crew();
    await openScout(fake);
    await openPanel(fake, scout.id);
    let approvalId = "";
    act(() => {
      approvalId = fake.browser.ask(scout.id, "Solve the captcha");
    });
    const callout = await within(panel()).findByRole("alert");
    expect(within(callout).getByText("Scout needs you in the browser")).toBeDefined();
    expect(within(callout).getByText("Solve the captcha")).toBeDefined();
    expect(within(panel()).queryByRole("button", { name: "Take control" })).not.toBeNull();

    const card = screen.getByRole("region", { name: "Scout asks: Solve the captcha" });
    fireEvent.change(within(card).getByRole("textbox"), { target: { value: "Not now" } });
    fireEvent.click(within(card).getByRole("button", { name: "I won't do it" }));
    await waitFor(() =>
      expect(called(fake, "approvals.answer")).toEqual([
        { approvalId, allow: false, note: "Not now" },
      ]),
    );
    expect(await screen.findByText("You didn't do it: Solve the captcha")).toBeDefined();
    await waitFor(() => expect(within(panel()).queryByRole("alert")).toBeNull());
  });
});
