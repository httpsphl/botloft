import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openNewCrewForm, renderApp } from "../../test/app";

afterEach(cleanup);

describe("the crew's automatic mode", () => {
  test("the new-crew form has the switch, and the crew is created in automatic mode", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    renderApp(fake);
    await crewOpened("Ops");

    const dialog = await openNewCrewForm();
    fireEvent.change(within(dialog).getByLabelText("Name"), { target: { value: "Sales" } });
    const toggle = within(dialog).getByRole("switch", { name: "Automatic mode" });
    expect(toggle.getAttribute("aria-checked")).toBe("false");
    fireEvent.click(toggle);
    fireEvent.click(within(dialog).getByRole("button", { name: "Create crew" }));

    await crewOpened("Chief");
    const create = fake.calls.find((call) => call.method === "crews.create");
    expect(create?.params).toMatchObject({ name: "Sales", autoMode: true });
    const crew = [...fake.crews.values()].find((item) => item.name === "Sales");
    expect(crew?.autoMode).toBe(true);
    const chief = [...fake.bots.values()].find((bot) => bot.crewId === crew?.id);
    expect(chief?.permissionMode).toBe("auto");
  });

  test("a crew made without it stays manual", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    renderApp(fake);
    await crewOpened("Ops");

    const dialog = await openNewCrewForm();
    fireEvent.change(within(dialog).getByLabelText("Name"), { target: { value: "Plain" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Create crew" }));

    await crewOpened("Chief");
    const create = fake.calls.find((call) => call.method === "crews.create");
    expect(create?.params).not.toHaveProperty("autoMode");
  });

  test("the header switch turns the whole crew to automatic, and keeps it for new agents", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    fake.addBot(crew.id, "Scout");
    renderApp(fake);
    await crewOpened("Ops");

    fireEvent.click(screen.getByRole("switch", { name: "Automatic mode" }));
    await waitFor(() =>
      expect(fake.calls.some((call) => call.method === "crews.setAutoMode")).toBe(true),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("switch", { name: "Automatic mode" }).getAttribute("aria-checked"),
      ).toBe("true"),
    );
    for (const bot of fake.activeBots(crew.id)) {
      expect(bot.permissionMode).toBe("auto");
    }

    const added = await fake.call("bots.create", {
      crewId: crew.id,
      name: "Later",
      role: "Helps",
      instructions: "",
    });
    expect(added.permissionMode).toBe("auto");
  });

  test("a narrow header shows short words, and the whole names are still what is read aloud", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    renderApp(fake);
    await crewOpened("Ops");

    const header = screen.getByRole("heading", { name: "Ops" }).closest("header") as HTMLElement;
    expect(within(header).getByRole("switch", { name: "Automatic mode" })).toBeTruthy();
    expect(within(header).getByRole("button", { name: "Agent agency" })).toBeTruthy();
    // The short forms are only for the eye: hidden from screen readers.
    for (const short of ["Automatic", "Agency"]) {
      expect(within(header).getByText(short).getAttribute("aria-hidden")).toBe("true");
    }
  });
});
