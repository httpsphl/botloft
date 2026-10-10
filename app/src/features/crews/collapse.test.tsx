import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";
import { resetCollapsed } from "./collapsed";

afterEach(() => {
  cleanup();
  resetCollapsed();
});

const inSidebar = (name: string) =>
  within(sidebar()).queryByRole("button", { name: new RegExp(`^${name},`) });

function twoCrews() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout");
  const writer = fake.addBot(ops.id, "Writer");
  const docs = fake.addCrew("Docs");
  const editor = fake.addBot(docs.id, "Editor");
  for (const bot of [scout, writer, editor]) {
    fake.setBotState(bot.id, "idle");
  }
  return { fake, ops, scout, writer, docs, editor };
}

describe("folding a crew in the sidebar", () => {
  test("hides its agents, keeps the open one, and is remembered", async () => {
    const { fake, ops, scout } = twoCrews();
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Writer");

    fireEvent.click(within(sidebar()).getByRole("button", { name: "Fold Ops" }));
    expect(inSidebar("Scout")).toBeNull();
    // The open conversation stays in view.
    expect(inSidebar("Writer")).not.toBeNull();
    expect(inSidebar("Editor")).not.toBeNull();
    expect(localStorage.getItem("botloft.collapsedCrews")).toContain(ops.id);

    // A bot waiting for the owner shows on the folded crew.
    act(() => fake.setBotState(scout.id, "needs_approval"));
    expect(within(sidebar()).getByRole("button", { name: /^Ops/ }).textContent).toContain(
      "an agent needs you",
    );

    const unfold = within(sidebar()).getByRole("button", { name: "Unfold Ops" });
    expect(unfold.getAttribute("aria-expanded")).toBe("false");
    fireEvent.click(unfold);
    expect(inSidebar("Scout")).not.toBeNull();
  });

  test("one button folds every crew, and unfolds them again", async () => {
    const { fake } = twoCrews();
    renderApp(fake);
    await crewOpened("Ops");

    fireEvent.click(within(sidebar()).getByRole("button", { name: "Fold all crews" }));
    expect(inSidebar("Scout")).toBeNull();
    expect(inSidebar("Editor")).toBeNull();

    fireEvent.click(within(sidebar()).getByRole("button", { name: "Unfold all crews" }));
    expect(inSidebar("Scout")).not.toBeNull();
    expect(inSidebar("Editor")).not.toBeNull();
  });
});

describe("the page of all crews", () => {
  test("opens from Crews on the left and leads to each crew", async () => {
    const { fake, editor } = twoCrews();
    fake.setBotState(editor.id, "busy");
    renderApp(fake);
    await crewOpened("Ops");

    fireEvent.click(within(sidebar()).getByRole("button", { name: "Crews" }));
    await crewOpened("Crews");
    const page = screen.getByRole("region", { name: "Crews" });
    expect(page.textContent).toContain("2 crews · 3 agents");
    const docs = within(page).getByRole("button", { name: /^Docs/ });
    expect(docs.textContent).toContain("1 working");
    expect(within(page).getByRole("button", { name: /^Ops/ }).textContent).toContain("All quiet");

    fireEvent.click(docs);
    await crewOpened("Docs");
  });
});
