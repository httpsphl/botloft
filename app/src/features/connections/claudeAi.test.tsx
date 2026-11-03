import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

async function openDetails(agent: "claude" | "codex" = "claude") {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.bot(scout.id).agent = agent;
  fake.setBotState(scout.id, "idle");
  fake.changes.connectors = {
    connectors: [
      { name: "Gmail", state: "connected" },
      { name: "Google Calendar", state: "connected" },
      { name: "Figma", state: "needs_auth" },
    ],
    checkedAt: 1,
  };
  const { host } = renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  fireEvent.click(await screen.findByRole("button", { name: "Show details" }));
  const panel = await screen.findByRole("complementary", { name: "About Scout" });
  return { fake, scout, panel, host };
}

describe("the claude.ai tools an agent may use (spec 34)", () => {
  test("lists the account's tools with a switch each, and gives one to the agent", async () => {
    const { fake, scout, panel, host } = await openDetails();
    const gmail = await within(panel).findByRole("switch", { name: "Scout can use Gmail" });
    expect(gmail.getAttribute("aria-checked")).toBe("false");
    expect(within(panel).getByText("Needs you to sign in")).toBeTruthy();

    fireEvent.click(within(panel).getByRole("switch", { name: "Scout can use Google Calendar" }));
    expect(
      (
        await within(panel).findByRole("switch", { name: "Scout can use Google Calendar" })
      ).getAttribute("aria-checked"),
    ).toBe("true");
    expect(fake.bot(scout.id).connectors).toEqual(["Google Calendar"]);

    fireEvent.click(within(panel).getByRole("button", { name: "Manage on claude.ai" }));
    await screen.findByRole("complementary", { name: "About Scout" });
    expect(host.opened).toContain("https://claude.ai/customize/connectors");
  });

  test("an agent on another engine has none", async () => {
    const { panel } = await openDetails("codex");
    await within(panel).findByText("Connected tools");
    expect(within(panel).queryByText("Tools from claude.ai")).toBeNull();
  });
});
