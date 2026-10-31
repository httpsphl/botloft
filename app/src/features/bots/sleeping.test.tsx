import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { setLocaleChoice } from "../../i18n";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";

afterEach(() => {
  act(() => setLocaleChoice("en"));
  cleanup();
});

describe("a sleeping agent", () => {
  test("is quiet in the list, sleeps as a mascot and says so in its chat", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "sleeping");
    renderApp(fake);
    await crewOpened("Ops");

    // In the list, like a free agent: no state, the mascot asleep.
    const row = within(sidebar()).getByRole("button", { name: /Scout/ });
    expect(within(row).queryByText("Sleeping")).toBeNull();
    expect(row.querySelector('[data-mood="sleeping"]')).not.toBeNull();

    // In its chat, the header says it sleeps and how it wakes.
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });
    const state = screen.getByText("Sleeping");
    expect(state.closest("[title]")?.getAttribute("title")).toMatch(/wakes by itself/);

    act(() => setLocaleChoice("pt-BR"));
    expect(await screen.findByText("Dormindo")).toBeDefined();
  });
});

describe("the time before agents sleep", () => {
  test("is chosen in General, Never included", async () => {
    const fake = new FakeBotloft();
    renderApp(fake);
    fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
    const page = screen.getByRole("region", { name: "Settings" });

    fireEvent.click(await within(page).findByRole("button", { name: /30 minutes/ }));
    fireEvent.click(screen.getByRole("option", { name: "1 hour" }));
    await waitFor(() => expect(fake.settings.sleepAfterMinutes).toBe(60));

    fireEvent.click(within(page).getByRole("button", { name: /1 hour/ }));
    fireEvent.click(screen.getByRole("option", { name: "Never" }));
    await waitFor(() => expect(fake.settings.sleepAfterMinutes).toBe(0));
  });
});
