import { act, cleanup, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { ChatItem } from "../../lib/protocol.gen";
import { crewOpened, openBot, renderApp } from "../../test/app";

// Counts how often finished tool lines render.
const renders = vi.hoisted(() => ({ tools: 0 }));
vi.mock("./ToolLines", () => ({
  ToolLines: ({ items }: { items: ChatItem[] }) => {
    renders.tools += 1;
    return <p>{items.length} tool calls</p>;
  },
}));

afterEach(cleanup);

describe("streaming", () => {
  test("text being written re-renders only the run it belongs to", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "idle");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const chat = await screen.findByRole("list", { name: "Messages" });
    act(() => {
      fake.chat.tool(scout.id, "Bash", { summary: "npm test", input: "{}" });
      fake.chat.turn(scout.id);
      fake.setBotState(scout.id, "busy");
    });
    const before = renders.tools;

    const pieces = ["The first ", "paragraph.\n\n", "The second ", "one."];
    for (const piece of pieces) {
      act(() => fake.chat.delta(scout.id, piece));
    }

    expect(within(chat).getByText("The second one.")).toBeDefined();
    expect(within(chat).getByText("The first paragraph.")).toBeDefined();
    expect(renders.tools).toBe(before);
  });
});
