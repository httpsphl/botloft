import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { CardBlock } from "../../lib/protocol.gen";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";
import { labelEvery, niceStep, scaleOf } from "./chartScale";

afterEach(cleanup);

/** A crew "Ops" with @analyst, its chat open. */
async function analystChat() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const analyst = fake.addBot(ops.id, "Analyst", "Crunches numbers");
  fake.setBotState(analyst.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Analyst");
  const chat = await screen.findByRole("list", { name: "Messages" });
  return { fake, analyst, chat };
}

const report: CardBlock[] = [
  { type: "text", text: "Up **8%**." },
  { type: "table", columns: ["Item", "Sold"], rows: [["Bread", "12"]] },
  {
    type: "chart",
    kind: "bar",
    labels: ["Mon", "Tue"],
    series: [
      { name: "Bread", values: [5, null] },
      { name: "Cake", values: [2, 3] },
    ],
    unit: "kg",
  },
  {
    type: "buttons",
    buttons: [
      { id: "publish", label: "Publish" },
      { id: "wait", label: "Wait" },
    ],
  },
];

describe("agent cards", () => {
  test("a card shows its blocks and a button answers its version", async () => {
    const { fake, analyst, chat } = await analystChat();
    act(() => {
      fake.chat.tool(analyst.id, "mcp__botloft__show_card", { status: "done" });
      fake.cards.show(analyst.id, "Weekly sales", report);
    });
    const card = within(chat).getByRole("region", { name: "Analyst's card: Weekly sales" });
    expect(within(card).getByText("8%").tagName).toBe("STRONG");
    expect(within(card).getByRole("cell", { name: "Bread" })).toBeDefined();
    expect(within(card).getByRole("img", { name: "Bar chart of Bread, Cake" })).toBeDefined();
    // Two series: a legend names them.
    expect(within(card).getAllByText("Cake").length).toBeGreaterThan(0);
    // The card stands for the call; no tool line repeats it.
    expect(within(chat).queryByText("Show you a card")).toBeNull();
    expect(within(sidebar()).getByText("Card: Weekly sales")).toBeDefined();

    fireEvent.click(within(card).getByRole("button", { name: "Show as a table" }));
    expect(within(card).getByRole("columnheader", { name: "kg" })).toBeDefined();
    expect(within(card).getByRole("cell", { name: "3 kg" })).toBeDefined();
    expect(within(card).getByRole("cell", { name: "—" })).toBeDefined();

    fireEvent.click(within(card).getByRole("button", { name: "Publish" }));
    await within(card).findByText("You chose “Publish”");
    const id = fake.chat.items.find((item) => item.body.kind === "card")?.id;
    expect(fake.calls.at(-1)).toEqual({
      method: "chat.act",
      params: { itemId: id, version: 1, button: "publish" },
    });
    expect(fake.conversation.messages.at(-1)?.body).toBe("Publish");
    expect(fake.conversation.messages.at(-1)?.replyTo).toEqual({
      itemId: id,
      text: "Weekly sales",
    });
    expect((within(card).getByRole("button", { name: "Wait" }) as HTMLButtonElement).disabled).toBe(
      true,
    );

    // Shown again: a new version takes an answer again.
    act(() => {
      fake.cards.show(analyst.id, "Weekly sales", report, id);
    });
    const again = within(chat).getByRole("region", { name: "Analyst's card: Weekly sales" });
    expect(within(again).queryByText("You chose “Publish”")).toBeNull();
    expect(
      (within(again).getByRole("button", { name: "Wait" }) as HTMLButtonElement).disabled,
    ).toBe(false);
  });

  test("a form waits for its required fields and sends one line per field", async () => {
    const { fake, analyst, chat } = await analystChat();
    act(() => {
      fake.cards.show(analyst.id, "Order", [
        {
          type: "form",
          submit: "Place the order",
          fields: [
            { id: "kg", label: "How many kg", kind: "number", options: [], required: true },
            {
              id: "flour",
              label: "Flour",
              kind: "choice",
              options: ["Wheat", "Rye"],
              required: false,
            },
            { id: "urgent", label: "Urgent", kind: "check", options: [], required: false },
          ],
        },
      ]);
    });
    const card = within(chat).getByRole("region", { name: "Analyst's card: Order" });
    const send = within(card).getByRole("button", { name: "Place the order" }) as HTMLButtonElement;
    expect(send.disabled).toBe(true);
    fireEvent.change(within(card).getByLabelText(/How many kg/), { target: { value: "25" } });
    fireEvent.change(within(card).getByLabelText("Flour"), { target: { value: "Rye" } });
    fireEvent.click(within(card).getByLabelText("Urgent"));
    expect(send.disabled).toBe(false);
    fireEvent.click(send);
    await within(card).findByText("You sent the form");
    expect(fake.conversation.messages.at(-1)?.body).toBe("How many kg: 25\nFlour: Rye\nUrgent: ✓");
  });
});

describe("the chart's scale", () => {
  test("takes round steps from zero", () => {
    expect(niceStep(0.7)).toBe(1);
    expect(niceStep(1.8)).toBe(2);
    expect(niceStep(23)).toBe(25);
    expect(niceStep(420)).toBe(500);
    expect(scaleOf([5, null, 3])).toEqual({ min: 0, max: 6, ticks: [0, 2, 4, 6] });
    expect(scaleOf([-3, 9]).ticks[0]).toBeLessThanOrEqual(-3);
    expect(scaleOf([null])).toEqual({ min: 0, max: 1, ticks: [0, 1] });
    expect(scaleOf([0.1, 0.3]).ticks).toEqual([0, 0.1, 0.2, 0.3]);
  });

  test("writes fewer labels when they would run together", () => {
    expect(labelEvery(6, 500)).toBe(1);
    expect(labelEvery(50, 500)).toBe(6);
  });
});
