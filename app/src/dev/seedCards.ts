// Dev only: a card the Analyst showed (spec 8.11), with a table, a chart,
// buttons and a form, to see every block in the preview.

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

export function seedCards(fake: FakeBotloft, analyst: BotId): void {
  const now = fake.now;
  fake.now = now - 3 * 60 * 1000;
  fake.chat.tool(analyst, "mcp__botloft__show_card", { status: "done" });
  fake.cards.show(analyst, "Flour prices, last 6 months", [
    {
      type: "text",
      text: "Prices went up **14%** since May. The mill on Rua Augusta is still the cheapest.",
    },
    {
      type: "chart",
      kind: "line",
      labels: ["May", "Jun", "Jul", "Aug", "Sep", "Oct"],
      series: [
        { name: "Mill A", values: [4.1, 4.2, 4.4, 4.5, 4.6, 4.7] },
        { name: "Mill B", values: [4.4, 4.5, null, 4.9, 5.0, 5.1] },
      ],
      unit: "R$/kg",
    },
    {
      type: "table",
      columns: ["Mill", "Price now", "Delivery"],
      rows: [
        ["Mill A", "4.70", "2 days"],
        ["Mill B", "5.10", "next day"],
      ],
    },
    {
      type: "buttons",
      buttons: [
        { id: "order-a", label: "Order from Mill A" },
        { id: "wait", label: "Wait a month" },
      ],
    },
  ]);
  fake.now = now - 2 * 60 * 1000;
  fake.chat.tool(analyst, "mcp__botloft__show_card", { status: "done" });
  fake.cards.show(analyst, "Order details", [
    {
      type: "form",
      submit: "Place the order",
      fields: [
        { id: "kg", label: "How many kg", kind: "number", options: [], required: true },
        {
          id: "flour",
          label: "Flour",
          kind: "choice",
          options: ["Wheat", "Whole wheat", "Rye"],
          required: true,
        },
        { id: "urgent", label: "Urgent", kind: "check", options: [], required: false },
        { id: "notes", label: "Notes", kind: "long_text", options: [], required: false },
      ],
    },
  ]);
  fake.now = now;
}
