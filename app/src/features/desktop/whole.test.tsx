import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { setLocaleChoice } from "../../i18n";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(() => {
  act(() => setLocaleChoice("en"));
  cleanup();
});

async function details() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  fireEvent.click(screen.getByRole("button", { name: /^Show details/ }));
  const about = screen.getByRole("complementary", { name: "About Scout" });
  return { fake, scout, about };
}

describe("the whole desktop", () => {
  test("is given only after the owner chooses a level and accepts the risks", async () => {
    const { fake, about } = await details();
    fireEvent.click(await within(about).findByRole("button", { name: "Give the whole desktop" }));
    const dialog = screen.getByRole("dialog", { name: "Give Scout your whole desktop?" });
    expect(within(dialog).getByText(/without asking you first/)).toBeDefined();
    expect(within(dialog).getByText(/covered in black/)).toBeDefined();
    const confirm = within(dialog).getByRole("button", { name: "Give it" }) as HTMLButtonElement;
    expect(confirm.disabled).toBe(true);

    fireEvent.click(within(dialog).getByRole("radio", { name: "Can see and use" }));
    fireEvent.click(within(dialog).getByRole("checkbox", { name: "I understand the risks" }));
    expect(confirm.disabled).toBe(false);
    fireEvent.click(confirm);
    await waitFor(() => expect(fake.desktop.grants[0]?.scope).toBe("desktop"));
    expect(fake.desktop.grants[0]?.level).toBe("act");
    expect(fake.desktop.grants[0]?.acceptedRisksAt).not.toBeNull();

    // Listed like any grant, with its options; the button goes.
    expect(await within(about).findByText("The whole desktop")).toBeDefined();
    expect(
      within(about).getByRole("switch", { name: "Real mouse and keyboard: The whole desktop" }),
    ).toBeDefined();
    expect(within(about).queryByRole("button", { name: "Give the whole desktop" })).toBeNull();
  }, 15_000);

  test("speaks the owner's language", async () => {
    const { about } = await details();
    act(() => setLocaleChoice("pt-BR"));
    fireEvent.click(await within(about).findByRole("button", { name: "Dar o desktop inteiro" }));
    expect(
      screen.getByRole("dialog", { name: "Dar a Scout o seu desktop inteiro?" }),
    ).toBeDefined();
  });
});
