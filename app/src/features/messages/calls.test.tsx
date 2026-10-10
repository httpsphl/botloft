import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import type { Delivery, Message } from "../../lib/protocol.gen";
import { liveCalls, nextChange, PICKED_UP_MS, withCall } from "../../store/calls";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

const NOW = 1_000_000_000_000;

function message(id: string, changes: Partial<Message> = {}): Message {
  return {
    id,
    crewId: "crw_1",
    fromKind: "bot",
    fromBotId: "bot_a",
    toBotId: "bot_b",
    kind: "note",
    body: "",
    taskId: null,
    routineId: null,
    questionId: null,
    attachments: [],
    createdAt: NOW,
    ...changes,
  } as Message;
}

function delivery(messageId: string, changes: Partial<Delivery> = {}): Delivery {
  return {
    id: `dlv_${messageId}`,
    messageId,
    botId: "bot_b",
    state: "sending",
    attempts: 0,
    nextAttemptAt: NOW,
    lastError: null,
    readAt: null,
    updatedAt: NOW,
    ...changes,
  } as Delivery;
}

describe("calls", () => {
  test("only an agent's message to another agent is a call", () => {
    const empty = { calls: {}, deliveries: {} };
    expect(withCall(empty, message("m1", { fromKind: "owner", fromBotId: null }))).toBeNull();
    expect(withCall(empty, message("m1", { toBotId: "bot_a" }))).toBeNull();
    expect(Object.keys(withCall(empty, message("m1"))?.calls ?? {})).toEqual(["m1"]);
  });

  test("ring until read, show the pick-up for a moment, and end when dead", () => {
    const calls = withCall({ calls: {}, deliveries: {} }, message("m1"), NOW)?.calls ?? {};
    const at = (deliveries: Record<string, Delivery>, now: number) =>
      liveCalls({ calls, deliveries }, now);

    expect(at({}, NOW + 1000)[0]?.readAt).toBeNull();
    const read = { m1: delivery("m1", { state: "sent", readAt: NOW + 2000 }) };
    expect(at(read, NOW + 3000)[0]?.readAt).toBe(NOW + 2000);
    expect(nextChange(at(read, NOW + 3000), NOW + 3000)).toBe(PICKED_UP_MS - 1000 + 1);
    expect(at(read, NOW + 2000 + PICKED_UP_MS + 1)).toEqual([]);
    expect(at({ m1: delivery("m1", { state: "dead" }) }, NOW + 1000)).toEqual([]);
  });

  test("one pill per pair of agents, the latest call", () => {
    let state = { calls: {}, deliveries: {} };
    state = { ...state, ...withCall(state, message("m1"), NOW) };
    state = { ...state, ...withCall(state, message("m2", { createdAt: NOW + 5 }), NOW) };
    state = {
      ...state,
      ...withCall(state, message("m3", { fromBotId: "bot_b", toBotId: "bot_a" }), NOW),
    };
    expect(liveCalls(state, NOW + 10).map((call) => call.messageId)).toEqual(["m3", "m2"]);
  });
});

describe("in the app", () => {
  test("the caller's chat and the crew page show the call until it is picked up", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    const writer = fake.addBot(ops.id, "Writer", "Writes");
    fake.setBotState(scout.id, "idle");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });

    let sent: ReturnType<typeof fake.conversation.say> | undefined;
    act(() => {
      sent = fake.conversation.say({ from: scout.id, to: writer.id, body: "Draft the report" });
    });
    const pills = () => screen.getByRole("list", { name: "Agents calling each other" });
    expect(within(pills()).getByText("Calling Writer")).toBeDefined();
    expect(within(pills()).getByText("Scout:")).toBeDefined();

    // The crew page shows it too.
    fireEvent.click(screen.getAllByRole("button", { name: "Ops" })[0] as HTMLElement);
    expect(
      within(await screen.findByRole("list", { name: "Agents calling each other" })).getByText(
        "Calling Writer",
      ),
    ).toBeDefined();

    act(() => {
      fake.conversation.read((sent as NonNullable<typeof sent>).delivery.id);
    });
    expect(within(pills()).getByText("Writer picked it up")).toBeDefined();
  });
});
