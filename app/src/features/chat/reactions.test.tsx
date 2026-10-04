import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

async function setup() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  fake.setBotState(scout.id, "idle");
  const said = fake.chat.reply(scout.id, "Acme signs annual.");
  fake.chat.turn(scout.id);
  fake.conversation.say({ from: writer.id, to: scout.id, body: "The draft is ready." });
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout, said };
}

function react(emoji: string) {
  fireEvent.click(screen.getByRole("button", { name: "React to what Scout wrote" }));
  const menu = screen.getByRole("menu", { name: "React to what Scout wrote" });
  fireEvent.click(within(menu).getByRole("menuitemradio", { name: emoji }));
}

describe("reacting to a reply", () => {
  test("puts the emoji under the reply, waiting for the next message, then seen", async () => {
    const { fake, scout, said } = await setup();
    // Only the bot's own replies take one.
    expect(screen.getAllByRole("button", { name: /^React to/ })).toHaveLength(1);

    react("👍");
    expect(await screen.findByText("Scout sees it with your next message")).toBeDefined();
    expect(fake.calls.at(-1)).toMatchObject({
      method: "reactions.set",
      params: { botId: scout.id, itemId: said.id, emoji: "👍" },
    });
    // The bot was not sent anything.
    expect(fake.calls.some((call) => call.method === "messages.send")).toBe(false);

    const field = screen.getByRole("textbox", { name: "Message to Scout" });
    fireEvent.change(field, { target: { value: "Next?" } });
    fireEvent.keyDown(field, { key: "Enter" });
    expect(await screen.findByText("Scout saw it")).toBeDefined();
  });

  test("another emoji replaces it, the same one or a click on it takes it off", async () => {
    await setup();
    react("👍");
    await screen.findByRole("button", { name: "👍 · Take the reaction off" });
    react("🎉");
    await screen.findByRole("button", { name: "🎉 · Take the reaction off" });
    react("🎉");
    await waitFor(() =>
      expect(screen.queryByRole("button", { name: /Take the reaction off/ })).toBeNull(),
    );
    react("❤️");
    fireEvent.click(await screen.findByRole("button", { name: "❤️ · Take the reaction off" }));
    await waitFor(() =>
      expect(screen.queryByRole("button", { name: /Take the reaction off/ })).toBeNull(),
    );
  });
});
