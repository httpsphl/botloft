import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { mainConversationOf } from "../../lib/conversations";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);
beforeEach(() => localStorage.clear());

/** A crew "Ops" with @scout, open on its chat. */
async function scoutOpen() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  fake.chat.reply(scout.id, "Hello from the main one");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByText("Hello from the main one");
  return { fake, scout };
}

/** The messages of the open chat. */
const messages = () => screen.getByRole("list", { name: "Messages" });
/** The agent's header, with its name. */
const header = () =>
  screen.getByRole("heading", { level: 1, name: "Scout" }).closest("header") as HTMLElement;

describe("conversations of an agent", () => {
  test("a new conversation opens its own chat, and what is sent goes to it", async () => {
    const { fake, scout } = await scoutOpen();
    expect(within(header()).getByText("Main")).toBeDefined();

    fireEvent.click(within(header()).getByRole("button", { name: "New conversation" }));
    await waitFor(() =>
      expect(within(messages()).queryByText("Hello from the main one")).toBeNull(),
    );
    const made = fake.calls.find((call) => call.method === "conversations.create");
    expect(made?.params).toEqual({ botId: scout.id });
    expect(within(header()).getAllByText("New conversation").length).toBeGreaterThan(0);

    const box = screen.getByLabelText("Message to Scout");
    fireEvent.change(box, { target: { value: "Plan the trip" } });
    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    await waitFor(() =>
      expect(fake.calls.some((call) => call.method === "messages.send")).toBe(true),
    );
    const sent = fake.calls.find((call) => call.method === "messages.send");
    const other = (sent?.params as { conversationId?: string } | undefined)?.conversationId;
    expect(other).toBeDefined();
    expect(other).not.toBe(mainConversationOf(scout.id));
    expect(await within(messages()).findByText("Plan the trip")).toBeDefined();

    // A reply in the main one does not show here; one in this one does.
    act(() => {
      fake.chat.reply(scout.id, "Main reply");
      fake.chat.reply(scout.id, "Trip reply", other);
    });
    expect(await within(messages()).findByText("Trip reply")).toBeDefined();
    expect(within(messages()).queryByText("Main reply")).toBeNull();
  });

  test("stop acts on the open conversation, which works while the main one rests", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout");
    fake.setBotState(scout.id, "idle");
    const other = fake.conversations.add(scout.id, "Trip plans");
    // The chat opens where the owner left it.
    localStorage.setItem("botloft.conversations", JSON.stringify({ [scout.id]: other.id }));
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    expect(await within(header()).findByText("Trip plans")).toBeDefined();
    expect(screen.queryByRole("button", { name: "Stop Scout" })).toBeNull();

    act(() => fake.conversations.set(other.id, { state: "busy" }));
    fireEvent.click(await screen.findByRole("button", { name: "Stop Scout" }));
    await waitFor(() =>
      expect(fake.calls).toContainEqual({
        method: "bots.interrupt",
        params: { botId: scout.id, conversationId: other.id },
      }),
    );
  });

  test("an agent that is not Claude Code has no other conversations", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout");
    fake.bot(scout.id).agent = "codex";
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });
    expect(within(header()).queryByRole("button", { name: "New conversation" })).toBeNull();
  });
});
