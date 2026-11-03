import { cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

/** "Ops" with Scout on Claude Code and Cody on Codex, and an account with three tools. */
async function setUp(scoutTools: string[] = []) {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const cody = fake.addBot(ops.id, "Cody", "Writes code");
  fake.bot(cody.id).agent = "codex";
  fake.setBotState(scout.id, "idle");
  fake.bot(scout.id).connectors = scoutTools;
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
  return { fake, scout, cody, host };
}

/** From Scout's details, "Choose tools" opens Settings > Tools. */
async function openTools() {
  openBot("Scout");
  fireEvent.click(await screen.findByRole("button", { name: "Show details" }));
  const details = await screen.findByRole("complementary", { name: "About Scout" });
  fireEvent.click(await within(details).findByRole("button", { name: "Choose tools" }));
  return screen.findByRole("region", { name: "Settings" });
}

describe("the tools of the owner's claude.ai account (spec 34.7)", () => {
  test("each is a card, and its dialog gives it to an agent", async () => {
    const { fake, scout, host } = await setUp();
    const settings = await openTools();
    const yours = await within(settings).findByRole("list", { name: "Yours" });
    const cards = within(yours).getAllByRole("button");
    expect(cards.map((card) => card.textContent?.split("claude.ai")[0])).toEqual([
      "Gmail",
      "Google Calendar",
      "Figma",
    ]);
    expect(within(yours).getByText("Needs you to sign in")).toBeTruthy();

    fireEvent.click(within(yours).getByRole("button", { name: /^Google Calendar/ }));
    const dialog = await screen.findByRole("dialog", { name: "Google Calendar" });
    // Only agents on Claude Code use them: Cody is not listed.
    expect(within(dialog).queryByRole("switch", { name: "Cody can use it" })).toBeNull();
    fireEvent.click(within(dialog).getByRole("switch", { name: "Scout can use it" }));
    expect(
      (await within(dialog).findByRole("switch", { name: "Scout can use it" })).getAttribute(
        "aria-checked",
      ),
    ).toBe("true");
    expect(fake.bot(scout.id).connectors).toEqual(["Google Calendar"]);

    fireEvent.click(within(dialog).getByRole("button", { name: "Manage on claude.ai" }));
    await screen.findByRole("dialog", { name: "Google Calendar" });
    expect(host.opened).toContain("https://claude.ai/customize/connectors");
  });

  test("Discover lists known tools the account lacks, each one a way to claude.ai", async () => {
    const { host } = await setUp();
    const settings = await openTools();
    fireEvent.click(within(settings).getByRole("tab", { name: "Discover" }));
    const discover = await within(settings).findByRole("list", { name: "Discover" });
    expect(
      within(discover).queryByRole("button", { name: "Connect Gmail on claude.ai" }),
    ).toBeNull();
    fireEvent.click(within(discover).getByRole("button", { name: "Connect Slack on claude.ai" }));
    await within(settings).findByRole("list", { name: "Discover" });
    expect(host.opened).toContain("https://claude.ai/customize/connectors");
  });

  test("an agent's details list its tools, and an agent on another engine has none", async () => {
    await setUp(["Google Calendar"]);
    openBot("Scout");
    fireEvent.click(await screen.findByRole("button", { name: "Show details" }));
    const details = await screen.findByRole("complementary", { name: "About Scout" });
    const list = await within(details).findByRole("list", { name: "Tools from claude.ai" });
    expect(list.textContent).toContain("Google Calendar");

    openBot("Cody");
    fireEvent.click(await screen.findByRole("button", { name: "Show details" }));
    const codyDetails = await screen.findByRole("complementary", { name: "About Cody" });
    await within(codyDetails).findByText("Connected tools");
    expect(within(codyDetails).queryByText("Tools from claude.ai")).toBeNull();
  });
});

describe("an agent asking for a tool of the account (spec 34.6)", () => {
  test("the card says what it is for and the owner allows it in the chat", async () => {
    const { fake, scout } = await setUp();
    fake.chat.ask(
      scout.id,
      "mcp__botloft__ask_connector",
      "Google Calendar",
      JSON.stringify({ name: "Google Calendar", why: "To see tomorrow's meetings." }),
    );
    openBot("Scout");
    const card = await screen.findByRole("region", { name: "Scout asks for Google Calendar" });
    expect(within(card).getByText("Scout wants to use your Google Calendar")).toBeTruthy();
    expect(within(card).getByText("To see tomorrow's meetings.")).toBeTruthy();
    fireEvent.click(within(card).getByRole("button", { name: "Allow" }));
    expect(await screen.findByText("You gave Google Calendar to Scout")).toBeTruthy();
    expect(fake.calls.find((each) => each.method === "approvals.answer")?.params).toMatchObject({
      allow: true,
    });
  });
});
