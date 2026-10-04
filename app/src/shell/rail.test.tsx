import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { crewOpened, openBot, rail, renderApp } from "../test/app";
import { prefs } from "./prefs";

afterEach(() => {
  cleanup();
  prefs.sidebar.reset();
});

const list = () => screen.getByRole("navigation", { name: "Crews", hidden: true });
const slot = () => list().closest(".sidebar-slot") as HTMLElement;
const place = (name: string) => within(rail()).getByRole("button", { name });

describe("the rail", () => {
  test("goes to each place and marks the one open", async () => {
    const fake = new FakeBotloft();
    fake.addBot(fake.addCrew("Ops").id, "Scout");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    expect(place("Home").getAttribute("aria-current")).toBeNull();

    fireEvent.click(place("Routines"));
    expect(place("Routines").getAttribute("aria-current")).toBe("page");
    fireEvent.click(place("Search"));
    expect(await screen.findByRole("searchbox")).toBeDefined();
    fireEvent.click(place("Home"));
    expect(place("Home").getAttribute("aria-current")).toBe("page");
    expect(place("Search").getAttribute("aria-current")).toBeNull();
  });

  test("stays while the list is hidden, with the count of questions, and Home brings the list back", async () => {
    const fake = new FakeBotloft();
    const scout = fake.addBot(fake.addCrew("Ops").id, "Scout");
    renderApp(fake);
    await crewOpened("Ops");
    fireEvent.click(screen.getByRole("button", { name: "Hide the bots list" }));
    expect(slot().dataset.open).toBe("false");
    act(() => {
      fake.questions.ask(scout.id, "Which client first?");
    });
    expect(place("Questions").textContent).toContain("1 question waits for you");

    fireEvent.click(place("Home"));
    expect(slot().dataset.open).toBe("true");
  });
});
