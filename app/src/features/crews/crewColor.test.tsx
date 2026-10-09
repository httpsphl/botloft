import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, renderApp, sidebar } from "../../test/app";

afterEach(cleanup);

describe("a crew's color", () => {
  test("the owner picks one in the rename dialog and it shows beside the crew", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    fake.addBot(crew.id, "Scout");
    renderApp(fake);
    await crewOpened("Ops");
    expect(within(sidebar()).queryByTestId("crew-color")).toBeNull();

    fireEvent.contextMenu(within(sidebar()).getByRole("button", { name: /^Ops/ }), {
      clientX: 100,
      clientY: 100,
    });
    const menu = screen.getByRole("menu", { name: "Actions for Ops" });
    fireEvent.click(within(menu).getByRole("menuitem", { name: "Rename" }));
    const dialog = await screen.findByRole("dialog");
    fireEvent.click(within(dialog).getByRole("button", { name: "Color #5EC8FF" }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Rename" }));

    const dot = await within(sidebar()).findByTestId("crew-color");
    expect(dot.style.background).toMatch(/#5ec8ff|rgb\(94, 200, 255\)/i);
    expect(fake.crew(crew.id).color).toBe("#5EC8FF");
  });
});
