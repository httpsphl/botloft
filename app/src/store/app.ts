// App state fed by the daemon: loaded on every (re)connection, then kept
// current by notifications (spec 11.3). `sync.ts` does the loading. Chats
// are not here: each open chat keeps its own page (spec 15.1).

import { createStore, type StoreApi } from "zustand/vanilla";
import type { BotloftApi, ConnectionState, ServerEvent } from "../lib/api";
import type {
  Bot,
  BotId,
  Crew,
  CrewId,
  Delivery,
  MessageId,
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
  selectedCrewId: CrewId | null;
  selectedBotId: BotId | null;
  selectCrew(crewId: CrewId | null): void;
  selectBot(botId: BotId): void;
  /** Applies a record a call returned, before its notification arrives. */
  putCrew(crew: Crew): void;
  putBot(bot: Bot): void;
  putDelivery(delivery: Delivery): void;
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
        viewTransition(() => set({ selectedCrewId: bot.crewId, selectedBotId: botId }));
      }
    },
    putCrew: (crew) => set((state) => withCrew(state, crew)),
    putBot: (bot) => set((state) => withBot(state, bot)),
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
    default:
      return null;
  }
}
