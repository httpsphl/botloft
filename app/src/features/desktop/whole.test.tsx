import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { setLocaleChoice } from "../../i18n";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp, settingsFromDetails } from "../../test/app";

afterEach(() => {
  act(() => setLocaleChoice("en"));
  cleanup();
});

/** Scout's details, then its "Your desktop" row: Settings > Permissions on it. */
async function desktopOfScout() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  const page = await settingsFromDetails("Scout", /^Your desktop/);
  return { fake, scout, page };
}

describe("the whole desktop", () => {
  test("is given to the agent the details came from, only after a level and the risks", async () => {
    const { fake, scout, page } = await desktopOfScout();
    fireEvent.click(await within(page).findByRole("button", { name: "Give an app" }));
    const dialog = screen.getByRole("dialog", { name: "Give an agent an app" });
    // On the agent the details came from.
    expect(within(dialog).getByRole("button", { name: /Scout · Ops/ })).toBeDefined();
    fireEvent.click(await within(dialog).findByRole("radio", { name: "The whole desktop" }));
    expect(within(dialog).getByText(/covered in black/)).toBeDefined();
    const confirm = within(dialog).getByRole("button", { name: "Give it" }) as HTMLButtonElement;
    expect(confirm.disabled).toBe(true);

    fireEvent.click(within(dialog).getByRole("radio", { name: "Can see and use" }));
    fireEvent.click(within(dialog).getByRole("checkbox", { name: "I understand the risks" }));
    expect(confirm.disabled).toBe(false);
    fireEvent.click(confirm);
    await waitFor(() => expect(fake.desktop.grants[0]?.scope).toBe("desktop"));
    expect(fake.desktop.grants[0]?.botId).toBe(scout.id);
    expect(fake.desktop.grants[0]?.level).toBe("act");
    expect(fake.desktop.grants[0]?.acceptedRisksAt).not.toBeNull();

    // Listed like any grant, with its options.
    expect(await within(page).findByText("The whole desktop")).toBeDefined();
    expect(
      within(page).getByRole("switch", { name: "Real mouse and keyboard: The whole desktop" }),
    ).toBeDefined();
  }, 15_000);

  test("speaks the owner's language", async () => {
    const { page } = await desktopOfScout();
    act(() => setLocaleChoice("pt-BR"));
    fireEvent.click(await within(page).findByRole("button", { name: "Dar um app" }));
    expect(screen.getByRole("dialog", { name: "Dar um app a um agente" })).toBeDefined();
    expect(await screen.findByRole("radio", { name: "O desktop inteiro" })).toBeDefined();
  });
});
