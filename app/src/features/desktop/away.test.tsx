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

async function opened() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  return { fake, scout };
}

describe("use while the owner is away", () => {
  test("is off, and goes on only after the owner checks that they understand the risks", async () => {
    const { fake, scout } = await opened();
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });
    act(() => {
      fake.desktop.grant(scout.id, NOTEPAD, "Notepad");
    });
    fireEvent.click(screen.getByRole("button", { name: /^Show details/ }));
    const details = screen.getByRole("complementary", { name: "About Scout" });
    const toggle = await within(details).findByRole("switch", {
      name: "While you are away: Notepad",
    });
    expect(toggle.getAttribute("aria-checked")).toBe("false");

    fireEvent.click(toggle);
    const dialog = screen.getByRole("dialog", {
      name: "Let Scout work in Notepad while you are away?",
    });
    expect(within(dialog).getByText(/Scout can make mistakes/)).toBeDefined();
    expect(within(dialog).getByText(/can try to trick Scout/)).toBeDefined();
    expect(within(dialog).getByText(/nobody watches as it happens/)).toBeDefined();
    expect(within(dialog).getByText(/password managers/)).toBeDefined();
    const confirm = within(dialog).getByRole("button", { name: "Turn on" }) as HTMLButtonElement;
    expect(confirm.disabled).toBe(true);
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(fake.desktop.grants[0]?.unattended).toBe(false);

    // Asked again, the box starts unchecked.
    fireEvent.click(toggle);
    const again = screen.getByRole("dialog");
    const box = within(again).getByRole("checkbox", { name: "I understand the risks" });
    expect((box as HTMLInputElement).checked).toBe(false);
    fireEvent.click(box);
    fireEvent.click(within(again).getByRole("button", { name: "Turn on" }));
    await waitFor(() => expect(fake.desktop.grants[0]?.unattended).toBe(true));
    expect(fake.desktop.grants[0]?.acceptedRisksAt).not.toBeNull();
    expect(toggle.getAttribute("aria-checked")).toBe("true");

    // Off at once, without asking.
    fireEvent.click(toggle);
    await waitFor(() => expect(fake.desktop.grants[0]?.unattended).toBe(false));
  }, 15_000);

  test("the owner hears, when back, what each agent used and opens its chat", async () => {
    const { fake, scout } = await opened();
    act(() => {
      fake.desktop.usedAway(scout.id, "Notepad");
    });
    const notice = await screen.findByRole("region", { name: "While you were away" });
    expect(within(notice).getByText("Scout used Notepad while you were away")).toBeDefined();

    fireEvent.click(within(notice).getByRole("button", { name: "See in the chat" }));
    await screen.findByRole("list", { name: "Messages" });

    fireEvent.click(within(notice).getByRole("button", { name: "Got it" }));
    await waitFor(() =>
      expect(screen.queryByRole("region", { name: "While you were away" })).toBeNull(),
    );
    expect(fake.desktop.away).toEqual([]);
  });

  test("speaks the owner's language", async () => {
    const { fake, scout } = await opened();
    act(() => setLocaleChoice("pt-BR"));
    act(() => {
      fake.desktop.usedAway(scout.id, "Excel");
    });
    expect(await screen.findByText("Scout usou o Excel enquanto você estava fora")).toBeDefined();
  });
});
