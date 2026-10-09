import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

async function agyBot() {
  const fake = new FakeBotloft();
  const crew = fake.addCrew("Ops");
  const bot = fake.addBot(crew.id, "Zed");
  fake.bot(bot.id).agent = "agy";
  fake.setBotState(bot.id, "idle");
  fake.setContext(bot.id, {
    usedTokens: 120_000,
    windowTokens: 1_000_000,
    autoCompactTokens: null,
    compacting: false,
    updatedAt: 0,
  });
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Zed");
  await screen.findByRole("list", { name: "Messages" });
  return fake;
}

describe("a bot that runs on another agent", () => {
  test("shows its models, not Claude's, and picks one", async () => {
    const fake = await agyBot();
    expect(screen.queryByRole("button", { name: /^Effort:/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /Plan default|Opus|Sonnet/ })).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Model: Default" }));
    const menu = await screen.findByRole("menu", { name: "Model" });
    fireEvent.click(
      await within(menu).findByRole("menuitemradio", { name: /Gemini 3.8 Flash \(High\)/ }),
    );
    await waitFor(() =>
      expect(fake.bots.values().next().value?.agentModel).toBe("gemini-3.8-flash-high"),
    );
  });

  test("shows how full the conversation is, without a way to compact it", async () => {
    await agyBot();
    fireEvent.click(screen.getByRole("button", { name: /Conversation space/ }));
    const panel = await screen.findByRole("dialog", { name: "Conversation space" });
    expect(within(panel).getByText(/An estimate/)).toBeDefined();
    expect(within(panel).queryByRole("button", { name: "Compact now" })).toBeNull();
  });
});
