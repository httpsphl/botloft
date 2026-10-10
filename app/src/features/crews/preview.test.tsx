import { cleanup, within } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, renderApp, sidebar } from "../../test/app";

afterEach(cleanup);

test("an agent's last reply shows in the list as plain words, a command as it is", async () => {
  const fake = new FakeBotloft();
  const crew = fake.addCrew("Bakery");
  const sales = fake.addBot(crew.id, "Sales");
  sales.lastActivity = {
    kind: "reply",
    text: "**Vendas do dia, 03/10:** 1 venda _nova_",
    tool: null,
    at: Date.now(),
  };
  const ops = fake.addBot(crew.id, "Ops");
  ops.lastActivity = { kind: "tool", text: "ls **/*.md", tool: "Bash", at: Date.now() };
  renderApp(fake);
  await crewOpened("Bakery");

  const list = within(sidebar());
  expect(list.getByText("Vendas do dia, 03/10: 1 venda nova")).toBeDefined();
  expect(list.getByText(/ls \*\*\/\*\.md/)).toBeDefined();
});
