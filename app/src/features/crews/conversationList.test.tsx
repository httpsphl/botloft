import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";

beforeEach(() => {
  localStorage.clear();
  // Replies from the start of time count; the app would skip older ones.
  localStorage.setItem("botloft.seenSince", "1");
});
afterEach(() => {
  cleanup();
  localStorage.clear();
});

/** Ops with @scout, which has a second conversation "Trip plans", and @writer. */
async function twoConversations() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout");
  const writer = fake.addBot(ops.id, "Writer");
  fake.setBotState(scout.id, "idle");
  fake.setBotState(writer.id, "idle");
  fake.chat.reply(scout.id, "Main answer");
  const trip = fake.conversations.add(scout.id, "Trip plans");
  fake.chat.reply(scout.id, "Lisbon it is", trip.id);
  renderApp(fake);
  await crewOpened("Ops");
  return { fake, scout, writer, trip };
}

const list = () => within(sidebar()).getByRole("list", { name: "Conversations with Scout" });
const messages = () => screen.getByRole("list", { name: "Messages" });

describe("the conversations in the sidebar", () => {
  test("an agent with more than one shows them under its line, and one opens", async () => {
    await twoConversations();
    const toggle = await within(sidebar()).findByRole("button", { name: "2 conversations" });
    expect(toggle.getAttribute("aria-expanded")).toBe("false");
    fireEvent.click(toggle);
    expect(within(list()).getByRole("button", { name: /^Main/ })).toBeDefined();

    fireEvent.click(within(list()).getByRole("button", { name: /^Trip plans/ }));
    expect(await within(messages()).findByText("Lisbon it is")).toBeDefined();
    expect(within(messages()).queryByText("Main answer")).toBeNull();
    const current = within(list()).getByRole("button", { name: /^Trip plans/ });
    expect(current.getAttribute("aria-current")).toBe("page");
  });

  test("a conversation is renamed and archived from its menu", async () => {
    const { fake, trip } = await twoConversations();
    fireEvent.click(await within(sidebar()).findByRole("button", { name: "2 conversations" }));
    fireEvent.contextMenu(within(list()).getByRole("button", { name: /^Trip plans/ }));
    fireEvent.click(await screen.findByRole("menuitem", { name: "Rename" }));
    const dialog = await screen.findByRole("dialog", { name: "Rename the conversation" });
    fireEvent.change(within(dialog).getByLabelText(/Name/), { target: { value: "Lisbon trip" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    expect(await within(list()).findByRole("button", { name: /^Lisbon trip/ })).toBeDefined();
    expect(fake.calls).toContainEqual({
      method: "conversations.rename",
      params: { conversationId: trip.id, title: "Lisbon trip" },
    });

    fireEvent.contextMenu(within(list()).getByRole("button", { name: /^Lisbon trip/ }));
    fireEvent.click(await screen.findByRole("menuitem", { name: "Archive" }));
    const ask = await screen.findByRole("dialog", { name: "Archive Lisbon trip?" });
    fireEvent.click(within(ask).getByRole("button", { name: "Archive" }));
    // One conversation left: no list.
    await waitFor(() =>
      expect(within(sidebar()).queryByRole("button", { name: "2 conversations" })).toBeNull(),
    );
  });

  test("a reply in another conversation marks the agent and that conversation", async () => {
    const { fake, scout, trip } = await twoConversations();
    // The owner read Scout's main conversation.
    openBot("Scout");
    await within(messages()).findByText("Main answer");
    openBot("Writer");
    await screen.findByRole("list", { name: "Messages" });
    act(() => {
      fake.chat.reply(scout.id, "Booked the hotel", trip.id);
    });
    const scoutRow = await within(sidebar()).findByRole("button", { name: /^Scout, .*unread/ });
    expect(scoutRow).toBeDefined();
    fireEvent.click(within(sidebar()).getByRole("button", { name: "2 conversations" }));
    expect(within(list()).getByRole("button", { name: "Trip plans, new reply" })).toBeDefined();
    expect(
      within(list()).getByRole("button", { name: /^Main/ }).getAttribute("aria-label"),
    ).not.toMatch(/new reply/);

    fireEvent.click(within(list()).getByRole("button", { name: /^Trip plans/ }));
    await waitFor(() =>
      expect(fake.calls).toContainEqual({
        method: "chat.read",
        params: { botId: scout.id, conversationId: trip.id },
      }),
    );
  });
});
