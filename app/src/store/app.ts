// App state fed by the daemon: loaded on every (re)connection, then kept
// current by notifications (spec 11.3). `sync.ts` does the loading. Chats
// are not here: each open chat keeps its own page (spec 15.1).

import { createStore, type StoreApi } from "zustand/vanilla";
import type { BotloftApi, ConnectionState, ServerEvent } from "../lib/api";
import type {
  Bot,
  BotId,
  BrowserState,
  Crew,
  CrewId,
  Delivery,
  MessageId,
  Routine,
  RoutineId,
  SystemStatus,
  Task,
  TaskId,
} from "../lib/protocol.gen";
import { viewTransition } from "../ui/motion";

export interface AppState {
  connection: ConnectionState;
  /** True once crews and bots arrived from the current connection. */
  loaded: boolean;
  loadError: string | null;
  system: SystemStatus | null;
  crews: Record<CrewId, Crew>;
  bots: Record<BotId, Bot>;
  /**
   * Deliveries by message (each message has one): the most recently
   * updated ones and every dead one, then every change.
   */
  deliveries: Record<MessageId, Delivery>;
  tasks: Record<TaskId, Task>;
  /** Every routine that is not archived (spec 20), then every change. */
  routines: Record<RoutineId, Routine>;
  /** The bots' browsers that are not closed (spec 21.7), then every change. */
  browsers: Record<BotId, BrowserState>;
  /** When the owner last had each bot open, for failed routine runs. */
  seenAt: Record<BotId, number>;
  selectedCrewId: CrewId | null;
  selectedBotId: BotId | null;
  selectCrew(crewId: CrewId | null): void;
  selectBot(botId: BotId): void;
  /** Applies a record a call returned, before its notification arrives. */
  putCrew(crew: Crew): void;
  putBot(bot: Bot): void;
  putDelivery(delivery: Delivery): void;
  putRoutine(routine: Routine): void;
  putBrowser(browser: BrowserState): void;
}

export type AppStore = StoreApi<AppState>;

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

/** Deliveries that gave up, newest first. */
export function deadDeliveries(state: AppState): Delivery[] {
  return Object.values(state.deliveries)
    .filter((delivery) => delivery.state === "dead")
    .sort((a, b) => b.updatedAt - a.updatedAt);
}

export function tasksOf(state: AppState, crewId: CrewId): Task[] {
  return Object.values(state.tasks)
    .filter((task) => task.crewId === crewId)
    .sort((a, b) => b.createdAt - a.createdAt || b.id.localeCompare(a.id));
}

export function createAppStore(api: BotloftApi): AppStore {
  return createStore<AppState>()((set, get) => ({
    connection: api.connection(),
    loaded: false,
    loadError: null,
    system: null,
    crews: {},
    bots: {},
    deliveries: {},
    tasks: {},
    routines: {},
    browsers: {},
    seenAt: loadSeen(),
    selectedCrewId: null,
    selectedBotId: null,
    selectCrew: (crewId) => {
      const { selectedCrewId, selectedBotId } = get();
      if (selectedCrewId !== crewId || selectedBotId !== null) {
        viewTransition(() => set({ selectedCrewId: crewId, selectedBotId: null }));
      }
    },
    selectBot: (botId) => {
      const bot = get().bots[botId];
      if (bot && get().selectedBotId !== botId) {
        const seenAt = { ...get().seenAt, [botId]: Date.now() };
        saveSeen(seenAt);
        viewTransition(() => set({ selectedCrewId: bot.crewId, selectedBotId: botId, seenAt }));
      }
    },
    putCrew: (crew) => set((state) => withCrew(state, crew)),
    putBot: (bot) => set((state) => withBot(state, bot)),
    putRoutine: (routine) => set((state) => withRoutine(state, routine)),
    putBrowser: (browser) => set((state) => withBrowser(state, browser)),
    putDelivery: (delivery) =>
      set((state) => ({ deliveries: { ...state.deliveries, [delivery.messageId]: delivery } })),
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
    case "chat.item": {
      const { item, activity } = event.params;
      const bot = state.bots[item.botId];
      if (!bot || !activity) {
        return null;
      }
      return { bots: { ...state.bots, [bot.id]: { ...bot, lastActivity: activity } } };
    }
    case "delivery.changed":
      return { deliveries: { ...state.deliveries, [event.params.messageId]: event.params } };
    case "task.changed":
      return { tasks: { ...state.tasks, [event.params.id]: event.params } };
    case "routine.changed":
      return withRoutine(state, event.params);
    case "browser.changed":
      return withBrowser(state, event.params);
    case "routine.run": {
      const routine = state.routines[event.params.routineId];
      const last = routine?.lastRun;
      // The notification of a newer run may arrive before its routine's.
      if (!routine || (last && last.id > event.params.id)) {
        return null;
      }
      return withRoutine(state, { ...routine, lastRun: event.params });
    }
    default:
      return null;
  }
}

/** Adds or replaces a browser; a closed one leaves the store. */
function withBrowser(state: AppState, browser: BrowserState): Partial<AppState> {
  const { [browser.botId]: _, ...rest } = state.browsers;
  return {
    browsers: browser.status === "closed" ? rest : { ...rest, [browser.botId]: browser },
  };
}

/** Adds or replaces a routine; an archived one leaves the store. */
function withRoutine(state: AppState, routine: Routine): Partial<AppState> {
  const { [routine.id]: _, ...rest } = state.routines;
  return { routines: routine.archivedAt === null ? { ...rest, [routine.id]: routine } : rest };
}

const SEEN_KEY = "botloft.seen";

/** When each bot was last open, remembered across restarts of the app. */
function loadSeen(): Record<BotId, number> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(SEEN_KEY) ?? "{}");
    return typeof parsed === "object" && parsed !== null ? (parsed as Record<BotId, number>) : {};
  } catch {
    return {};
  }
}

function saveSeen(seen: Record<BotId, number>): void {
  try {
    localStorage.setItem(SEEN_KEY, JSON.stringify(seen));
  } catch {
    // Storage may be off; the mark just shows again after a restart.
  }
}

/** The routines of `botId`, oldest first. */
export function routinesOf(state: AppState, botId: BotId): Routine[] {
  return Object.values(state.routines)
    .filter((routine) => routine.botId === botId)
    .sort(byCreation);
}

/** Routines whose last run failed after the owner last had their bot open. */
export function unseenFailures(state: AppState): Routine[] {
  return Object.values(state.routines).filter((routine) => {
    const last = routine.lastRun;
    return (
      last?.status === "failed" &&
      routine.botId !== state.selectedBotId &&
      (last.finishedAt ?? 0) > (state.seenAt[routine.botId] ?? 0)
    );
  });
}
