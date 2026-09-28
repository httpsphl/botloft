// App state fed by the daemon: loaded on every (re)connection, then kept
// current by notifications (spec 11.3).

import { createStore, type StoreApi } from "zustand/vanilla";
import { type BotloftApi, type ConnectionState, errorText, type ServerEvent } from "../lib/api";
import type { Bot, BotId, Crew, CrewId, SystemStatus } from "../lib/protocol.gen";

export interface AppState {
  connection: ConnectionState;
  /** True once crews and bots arrived from the current connection. */
  loaded: boolean;
  loadError: string | null;
  system: SystemStatus | null;
  crews: Record<CrewId, Crew>;
  bots: Record<BotId, Bot>;
  selectedCrewId: CrewId | null;
  selectedBotId: BotId | null;
  selectCrew(crewId: CrewId | null): void;
  selectBot(botId: BotId): void;
  /** Applies a record a call returned, before its notification arrives. */
  putCrew(crew: Crew): void;
  putBot(bot: Bot): void;
}

export type AppStore = StoreApi<AppState>;

/** How often `system.status` is refreshed; it has no notification. */
const STATUS_POLL_MS = 15_000;

const byCreation = <T extends { createdAt: number; id: string }>(a: T, b: T) =>
  a.createdAt - b.createdAt || a.id.localeCompare(b.id);

export function crewList(state: AppState): Crew[] {
  return Object.values(state.crews).sort(byCreation);
}

export function botsOf(state: AppState, crewId: CrewId): Bot[] {
  return Object.values(state.bots)
    .filter((bot) => bot.crewId === crewId)
    .sort(byCreation);
}

export function createAppStore(api: BotloftApi): AppStore {
  return createStore<AppState>()((set, get) => ({
    connection: api.connection(),
    loaded: false,
    loadError: null,
    system: null,
    crews: {},
    bots: {},
    selectedCrewId: null,
    selectedBotId: null,
    selectCrew: (crewId) => set({ selectedCrewId: crewId, selectedBotId: null }),
    selectBot: (botId) => {
      const bot = get().bots[botId];
      if (bot) {
        set({ selectedCrewId: bot.crewId, selectedBotId: botId });
      }
    },
    putCrew: (crew) => set((state) => withCrew(state, crew)),
    putBot: (bot) => set((state) => withBot(state, bot)),
  }));
}

function withCrew(state: AppState, crew: Crew): Partial<AppState> {
  if (crew.archivedAt === null) {
    return { crews: { ...state.crews, [crew.id]: crew } };
  }
  const { [crew.id]: _gone, ...crews } = state.crews;
  const bots = Object.fromEntries(
    Object.entries(state.bots).filter(([, bot]) => bot.crewId !== crew.id),
  );
  const selected = state.selectedCrewId === crew.id;
  return {
    crews,
    bots,
    selectedCrewId: selected ? null : state.selectedCrewId,
    selectedBotId: selected ? null : state.selectedBotId,
  };
}

function withBot(state: AppState, bot: Bot): Partial<AppState> {
  if (bot.archivedAt === null) {
    return { bots: { ...state.bots, [bot.id]: bot } };
  }
  const { [bot.id]: _gone, ...bots } = state.bots;
  return { bots, selectedBotId: state.selectedBotId === bot.id ? null : state.selectedBotId };
}

export function applyEvent(state: AppState, event: ServerEvent): Partial<AppState> | null {
  switch (event.name) {
    case "crew.changed":
      return withCrew(state, event.params);
    case "bot.changed":
      return withBot(state, event.params);
    case "bot.state": {
      const bot = state.bots[event.params.botId];
      if (!bot) {
        return null;
      }
      const { state: botState, generation } = event.params;
      return { bots: { ...state.bots, [bot.id]: { ...bot, state: botState, generation } } };
    }
    default:
      return null;
  }
}

/** Keeps `store` in sync with the daemon until the returned function runs. */
export function syncStore(store: AppStore, api: BotloftApi): () => void {
  let poll: ReturnType<typeof setInterval> | undefined;
  let alive = true;

  const refreshStatus = () => {
    api.call("system.status").then(
      (system) => alive && store.setState({ system }),
      () => {},
    );
  };

  // Each part is applied as soon as it arrives: a notification that comes
  // later in the stream is newer than the list and must win.
  const load = () => {
    store.setState({ loaded: false, loadError: null });
    const fail = (error: unknown) => alive && store.setState({ loadError: errorText(error) });
    refreshStatus();
    const crews = api.call("crews.list").then((list) => {
      if (alive) {
        store.setState({ crews: Object.fromEntries(list.map((crew) => [crew.id, crew])) });
      }
    });
    const bots = api.call("bots.list", {}).then((list) => {
      if (alive) {
        store.setState({ bots: Object.fromEntries(list.map((bot) => [bot.id, bot])) });
      }
    });
    Promise.all([crews, bots]).then(() => {
      if (!alive) {
        return;
      }
      const state = store.getState();
      const valid = state.selectedCrewId !== null && state.crews[state.selectedCrewId];
      if (!valid) {
        state.selectCrew(crewList(state)[0]?.id ?? null);
      }
      store.setState({ loaded: true });
    }, fail);
  };

  const onConnection = (connection: ConnectionState) => {
    store.setState({ connection });
    clearInterval(poll);
    if (connection.kind === "open") {
      load();
      poll = setInterval(refreshStatus, STATUS_POLL_MS);
    }
  };

  const unsubscribeEvents = api.subscribe((event) => {
    const change = applyEvent(store.getState(), event);
    if (change) {
      store.setState(change);
    }
  });
  const unsubscribeConnection = api.onConnection(onConnection);
  onConnection(api.connection());

  return () => {
    alive = false;
    clearInterval(poll);
    unsubscribeEvents();
    unsubscribeConnection();
  };
}
