import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { setLocaleChoice } from "../../i18n";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(() => {
  act(() => setLocaleChoice("en"));
  cleanup();
});

const NOTEPAD = "C:\\Windows\\notepad.exe";

async function openScout() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout };
}

const details = () => screen.getByRole("complementary", { name: "About Scout" });

describe("the real mouse and keyboard", () => {
  test("are off, and turning them on says first what changes", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.desktop.grant(scout.id, NOTEPAD, "Notepad", "act");
    });
    fireEvent.click(screen.getByRole("button", { name: /^Show details/ }));
    const toggle = await within(details()).findByRole("switch", {
      name: "Real mouse and keyboard: Notepad",
    });
    expect(toggle.getAttribute("aria-checked")).toBe("false");

    fireEvent.click(toggle);
    const dialog = screen.getByRole("dialog", {
      name: "Let Scout use your real mouse and keyboard in Notepad?",
    });
    expect(within(dialog).getByText(/Move the mouse or press a key and it stops/)).toBeDefined();
    expect(within(dialog).getByText(/Ctrl\+Alt\+Esc/)).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(fake.desktop.grants[0]?.realInput).toBe(false);

    fireEvent.click(toggle);
    fireEvent.click(screen.getByRole("button", { name: "Turn on" }));
    await waitFor(() => expect(fake.desktop.grants[0]?.realInput).toBe(true));
    expect(toggle.getAttribute("aria-checked")).toBe("true");

    // Off at once, without asking.
    fireEvent.click(toggle);
    await waitFor(() => expect(fake.desktop.grants[0]?.realInput).toBe(false));
  });

  test("are not offered where the bot may only see", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.desktop.grant(scout.id, NOTEPAD, "Notepad");
    });
    fireEvent.click(screen.getByRole("button", { name: /^Show details/ }));
    await within(details()).findByText("Can see");
    expect(within(details()).queryByRole("switch", { name: /Real mouse/ })).toBeNull();
  });

  test("the daemon hears the owner's language when it changes, for its notice", async () => {
    const { fake } = await openScout();
    act(() => setLocaleChoice("pt-BR"));
    await waitFor(() => expect(fake.locale).toBe("pt-BR"));
  });
});
