import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { crewOpened, renderApp } from "../test/app";
import { prefs } from "./prefs";

afterEach(() => {
  cleanup();
  prefs.sidebar.reset();
});

const list = () => screen.getByRole("navigation", { name: "Crews", hidden: true });
const slot = () => list().closest(".sidebar-slot") as HTMLElement;

async function oneCrew() {
  const fake = new FakeBotloft();
  const crew = fake.addCrew("Ops");
  const scout = fake.addBot(crew.id, "Scout");
  renderApp(fake);
  await crewOpened("Ops");
  return { fake, scout };
}

describe("hiding the agents list", () => {
  test("the title bar button hides the list and brings it back, and it is remembered", async () => {
    await oneCrew();
    expect(slot().dataset.open).toBe("true");
    fireEvent.click(screen.getByRole("button", { name: "Hide the agents list" }));
    expect(slot().dataset.open).toBe("false");
    expect(slot().hasAttribute("inert")).toBe(true);
    expect(localStorage.getItem("botloft.sidebar")).toBe("false");
    fireEvent.click(screen.getByRole("button", { name: "Show the agents list" }));
    expect(slot().dataset.open).toBe("true");
    expect(slot().hasAttribute("inert")).toBe(false);
  });

  test("Ctrl+B does the same", async () => {
    await oneCrew();
    fireEvent.keyDown(window, { key: "b", ctrlKey: true });
    expect(slot().dataset.open).toBe("false");
    fireEvent.keyDown(window, { key: "B", ctrlKey: true });
    expect(slot().dataset.open).toBe("true");
    fireEvent.keyDown(window, { key: "b" });
    expect(slot().dataset.open).toBe("true");
  });

  test("with the list hidden, a dot on the button says an agent is waiting", async () => {
    const { fake, scout } = await oneCrew();
    fireEvent.click(screen.getByRole("button", { name: "Hide the agents list" }));
    const button = screen.getByRole("button", { name: "Show the agents list" });
    expect(button.parentElement?.querySelector(".live-dot")).toBeNull();
    fake.setBotState(scout.id, "needs_approval");
    await waitFor(() => expect(button.parentElement?.querySelector(".live-dot")).not.toBeNull());
  });
});
