import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

const NOTEPAD = "C:\\Windows\\notepad.exe";

async function openScout() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout };
}

const details = () => screen.getByRole("complementary", { name: "About Scout" });

describe("the desktop", () => {
  test("a request names the app, says why and what seeing means, and allowing answers it", async () => {
    const { fake, scout } = await openScout();
    let approvalId = "";
    act(() => {
      const item = fake.chat.ask(
        scout.id,
        "mcp__botloft__desktop",
        "Notepad",
        JSON.stringify({
          app: "Notepad",
          path: NOTEPAD,
          level: "see",
          why: "To read the shopping list",
        }),
      );
      approvalId = item.body.kind === "approval" ? item.body.approvalId : "";
    });
    const card = screen.getByRole("region", { name: "Scout asks to see Notepad" });
    expect(within(card).getByText("To read the shopping list")).toBeDefined();
    expect(within(card).getByText(NOTEPAD)).toBeDefined();
    expect(within(card).getByText(/Never your passwords/)).toBeDefined();
    // No "Always allow" here: allowing is for good.
    expect(within(card).queryByRole("button", { name: /Always allow/ })).toBeNull();
    fireEvent.click(within(card).getByRole("button", { name: "Allow" }));
    await waitFor(() =>
      expect(fake.calls).toContainEqual({
        method: "approvals.answer",
        params: { approvalId, allow: true },
      }),
    );
    expect(await screen.findByText("Scout can see Notepad")).toBeDefined();
  });

  test("a request to use an app says what using it means", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.chat.ask(
        scout.id,
        "mcp__botloft__desktop",
        "Notepad",
        JSON.stringify({ app: "Notepad", path: NOTEPAD, level: "act", why: "To save the list" }),
      );
    });
    const card = screen.getByRole("region", { name: "Scout asks to use Notepad" });
    expect(within(card).getByText(/without moving your mouse/)).toBeDefined();
    expect(within(card).getByText(/Never in password fields/)).toBeDefined();
    fireEvent.click(within(card).getByRole("button", { name: "Deny" }));
    expect(await screen.findByText("Scout may not use Notepad")).toBeDefined();
  });

  test("the agent's details list what it may see on the desktop, and it can be taken back", async () => {
    const { fake, scout } = await openScout();
    fireEvent.click(screen.getByRole("button", { name: /^Show details/ }));
    expect(
      await within(details()).findByText(
        "Scout cannot see any app on your computer. It asks in the chat the first time it needs one.",
      ),
    ).toBeDefined();
    act(() => {
      fake.desktop.grant(scout.id, NOTEPAD, "Notepad");
    });
    expect(await within(details()).findByText("Notepad")).toBeDefined();
    expect(within(details()).getByText("Can see")).toBeDefined();
    fireEvent.click(within(details()).getByRole("button", { name: "Take back Notepad" }));
    expect(
      await within(details()).findByText(
        "Scout cannot see any app on your computer. It asks in the chat the first time it needs one.",
      ),
    ).toBeDefined();
    expect(fake.desktop.grants).toHaveLength(0);
  });
});
