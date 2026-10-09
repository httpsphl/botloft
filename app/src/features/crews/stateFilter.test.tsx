import { act, cleanup, fireEvent, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, renderApp, sidebar } from "../../test/app";
import { resetStateFilter } from "./stateFilter";

afterEach(() => {
  cleanup();
  resetStateFilter();
});

describe("state filters in the conversation list", () => {
  test("a chip narrows the list to the bots in that state, with counts", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const calm = fake.addCrew("Calm");
    const scout = fake.addBot(ops.id, "Scout");
    const writer = fake.addBot(ops.id, "Writer");
    const idle = fake.addBot(calm.id, "Idle one");
    fake.setBotState(scout.id, "busy");
    fake.setBotState(writer.id, "needs_approval");
    fake.setBotState(idle.id, "idle");
    renderApp(fake);
    await crewOpened("Ops");

    const list = within(sidebar());
    expect(list.getByRole("button", { name: /^Working\s*1$/ })).toBeTruthy();
    expect(list.getByRole("button", { name: /^All\s*3$/ })).toBeTruthy();

    fireEvent.click(list.getByRole("button", { name: /^Needs you/ }));
    expect(list.queryByRole("button", { name: /^Scout,/ })).toBeNull();
    expect(list.getByRole("button", { name: /^Writer,/ })).toBeTruthy();
    // A crew with nothing in that state steps aside.
    expect(list.queryByText("Calm")).toBeNull();

    act(() => fake.setBotState(scout.id, "needs_approval"));
    expect(list.getByRole("button", { name: /^Scout,/ })).toBeTruthy();

    fireEvent.click(list.getByRole("button", { name: /^All/ }));
    expect(list.getByText("Calm")).toBeTruthy();
  });
});
