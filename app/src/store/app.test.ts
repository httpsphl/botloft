import { describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { activityOf, botsOf, createAppStore, crewList } from "./app";
import { syncStore } from "./sync";

async function synced(fake: FakeBotloft) {
  const store = createAppStore(fake);
  const stop = syncStore(store, fake);
  await settle();
  return { store, stop };
}

/** Lets resolved calls apply. */
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("app store", () => {
  test("loads crews, agents and the system status, and selects the first crew", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const research = fake.addCrew("Research");
    fake.addBot(research.id, "Scout");
    const { store } = await synced(fake);
    const state = store.getState();
    expect(state.loaded).toBe(true);
    expect(state.system?.claudeVersion).toBe("2.1.284");
    expect(crewList(state).map((crew) => crew.name)).toEqual(["Ops", "Research"]);
    expect(botsOf(state, research.id).map((bot) => bot.handle)).toEqual(["scout"]);
    expect(state.selectedCrewId).toBe(ops.id);
  });

  test("a chat item moves the agent's list line, not the agent", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    const { store } = await synced(fake);
    const before = store.getState().bots[scout.id];

    fake.chat.reply(scout.id, "Found three sources.");

    const state = store.getState();
    expect(state.bots[scout.id]).toBe(before);
    expect(activityOf(state, scout.id)).toMatchObject({ text: "Found three sources." });
  });

  test("follows state changes and archiving", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    const writer = fake.addBot(crew.id, "Writer");
    const { store } = await synced(fake);

    fake.setBotState(scout.id, "idle", 7);
    expect(store.getState().bots[scout.id]).toMatchObject({ state: "idle", generation: 7 });

    store.getState().selectBot(scout.id);
    await fake.call("bots.archive", { botId: scout.id });
    expect(store.getState().bots[scout.id]).toBeUndefined();
    expect(store.getState().selectedBotId).toBeNull();
    expect(store.getState().selectedCrewId).toBe(crew.id);

    await fake.call("crews.archive", { crewId: crew.id });
    expect(store.getState().crews).toEqual({});
    expect(store.getState().bots[writer.id]).toBeUndefined();
    expect(store.getState().selectedCrewId).toBeNull();
  });

  test("a deleted agent leaves with its routines, tasks and deliveries", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    const writer = fake.addBot(crew.id, "Writer");
    await fake.call("crews.setLead", { crewId: crew.id, botId: scout.id });
    const routine = await fake.call("routines.create", {
      botId: scout.id,
      name: "Morning",
      prompt: "Summarize",
      schedule: { kind: "interval", minutes: 60 },
      timezone: "UTC",
    });
    const { store } = await synced(fake);
    const task = fake.conversation.task(writer.id, scout.id);
    const sent = await fake.call("messages.send", { botId: scout.id, body: "Hello" });
    store.getState().selectBot(scout.id);
    expect(store.getState().tasks[task.id]).toBeDefined();
    expect(store.getState().deliveries[sent.id]).toBeDefined();

    expect(await fake.call("bots.delete", { botId: scout.id })).toEqual({
      botId: scout.id,
      crewId: crew.id,
    });
    const state = store.getState();
    expect(state.bots[scout.id]).toBeUndefined();
    expect(state.bots[writer.id]).toBeDefined();
    expect(state.routines[routine.id]).toBeUndefined();
    expect(state.tasks[task.id]).toBeUndefined();
    expect(state.deliveries[sent.id]).toBeUndefined();
    expect(state.crews[crew.id]?.leadBotId).toBeNull();
    expect(state.selectedBotId).toBeNull();
    expect(state.selectedCrewId).toBe(crew.id);
    await expect(fake.call("bots.delete", { botId: scout.id })).rejects.toThrow();
  });

  test("a deleted crew leaves with its agents, archived or not", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const docs = fake.addCrew("Docs");
    const scout = fake.addBot(ops.id, "Scout");
    const old = fake.addBot(ops.id, "Old");
    const editor = fake.addBot(docs.id, "Editor");
    await fake.call("bots.archive", { botId: old.id });
    const { store } = await synced(fake);
    store.getState().selectBot(scout.id);

    await fake.call("crews.delete", { crewId: ops.id });
    const state = store.getState();
    expect(Object.keys(state.crews)).toEqual([docs.id]);
    expect(Object.keys(state.bots)).toEqual([editor.id]);
    expect(state.selectedCrewId).toBeNull();
    expect(state.selectedBotId).toBeNull();
    expect(fake.bots.has(old.id)).toBe(false);

    // A delete the call returned is applied without waiting for the news.
    store.getState().dropBot(editor.id);
    store.getState().dropCrew(docs.id);
    expect(store.getState().bots).toEqual({});
    expect(store.getState().crews).toEqual({});
  });

  test("reloads everything when the connection comes back", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const { store } = await synced(fake);
    fake.setConnection({ kind: "waiting", retryAt: 0 });
    // Missed while disconnected: no notification reaches the store.
    const missed = fake.addBot(crew.id, "Scout");
    expect(store.getState().bots[missed.id]).toBeUndefined();
    fake.setConnection({ kind: "open", daemonVersion: "0.1.0" });
    await settle();
    expect(store.getState().bots[missed.id]?.name).toBe("Scout");
  });

  test("stops listening when unsynced", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const { store, stop } = await synced(fake);
    stop();
    fake.addBot(crew.id, "Scout");
    expect(Object.keys(store.getState().bots)).toHaveLength(0);
  });
});
