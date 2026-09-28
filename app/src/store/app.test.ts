import { describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { botsOf, createAppStore, crewList } from "./app";
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
  test("loads crews, bots and the system status, and selects the first crew", async () => {
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
