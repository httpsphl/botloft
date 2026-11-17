import { cleanup, screen } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

describe("an agent searching its past conversations (spec 8.8)", () => {
  test("the tool line names the search and the words", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "idle");
    const call = fake.chat.tool(scout.id, "mcp__botloft__search_history", {
      summary: "Lisbon budget",
      input: JSON.stringify({ query: "Lisbon budget" }),
    });
    fake.chat.finish(call, JSON.stringify({ hits: [] }), false);

    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const line = await screen.findByRole("button", { name: /Search past conversations/ });
    expect(line.textContent).toContain("Lisbon budget");
  });
});
