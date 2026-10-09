import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, renderApp, sidebar } from "../../test/app";
import { moveBeside, moveBy, resetCrewOrder } from "./crewOrder";

beforeEach(() => {
  localStorage.removeItem("botloft.collapsedCrews");
  resetCrewOrder();
});
afterEach(() => {
  cleanup();
  resetCrewOrder();
});

/** The crews' names in the sidebar, top to bottom. */
const names = () =>
  [...sidebar().querySelectorAll<HTMLElement>(":scope [data-crew-id]")].map(
    (item) => item.querySelector("button[aria-current], button.font-semibold")?.textContent ?? "",
  );

async function threeCrews() {
  const fake = new FakeBotloft();
  fake.addCrew("Alpha");
  fake.addCrew("Beta");
  fake.addCrew("Gamma");
  renderApp(fake);
  await crewOpened("Alpha");
  return fake;
}

describe("the order of crews", () => {
  test("moving an id beside another", () => {
    expect(moveBeside(["a", "b", "c"], "a", "c", true)).toEqual(["b", "c", "a"]);
    expect(moveBeside(["a", "b", "c"], "c", "a", false)).toEqual(["c", "a", "b"]);
    expect(moveBeside(["a", "b", "c"], "b", "b", true)).toEqual(["a", "b", "c"]);
    expect(moveBy(["a", "b", "c"], "a", -1)).toEqual(["a", "b", "c"]);
    expect(moveBy(["a", "b", "c"], "a", 1)).toEqual(["b", "a", "c"]);
  });

  test("the menu moves a crew up and down, and it stays after a reload", async () => {
    await threeCrews();
    const open = (name: string) =>
      fireEvent.contextMenu(within(sidebar()).getByRole("button", { name }), {
        clientX: 100,
        clientY: 100,
      });
    open("Gamma");
    const menu = screen.getByRole("menu", { name: "Actions for Gamma" });
    expect(within(menu).queryByRole("menuitem", { name: "Move down" })).toBeNull();
    fireEvent.click(within(menu).getByRole("menuitem", { name: "Move up" }));
    expect(names().map((n) => n.trim())).toEqual(["Alpha", "Gamma", "Beta"]);
    expect(JSON.parse(localStorage.getItem("botloft.crewOrder") ?? "[]")).toHaveLength(3);
  });

  test("dragging a crew's name drops it where the pointer lets go", async () => {
    await threeCrews();
    const items = [...sidebar().querySelectorAll<HTMLElement>("[data-crew-id]")];
    items.forEach((item, index) => {
      item.getBoundingClientRect = () =>
        ({ top: index * 100, bottom: index * 100 + 100, height: 100 }) as DOMRect;
    });
    const handle = within(sidebar()).getByRole("button", { name: "Alpha" });
    fireEvent.pointerDown(handle, { button: 0, clientX: 20, clientY: 10 });
    act(() => {
      fireEvent.pointerMove(window, { clientX: 20, clientY: 150 });
    });
    act(() => {
      fireEvent.pointerMove(window, { clientX: 20, clientY: 290 });
    });
    act(() => {
      fireEvent.pointerUp(window);
    });
    expect(names().map((n) => n.trim())).toEqual(["Beta", "Gamma", "Alpha"]);
    // The click that ends a drag does not open the crew.
    fireEvent.click(handle);
  });
});
