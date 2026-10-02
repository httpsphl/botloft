// App state fed by the daemon: loaded on every (re)connection, then kept
// current by notifications (spec 11.3). `sync.ts` does the loading. Chats
// are not here: each open chat keeps its own page (spec 15.1).

import { createStore, type StoreApi } from "zustand/vanilla";
import type { BotloftApi, ConnectionState, ServerEvent } from "../lib/api";
import type {
  Activity,
  Bot,
  BotId,
  BrowserState,
  Crew,
  CrewId,
  Delivery,
  MessageId,
  Routine,
  RoutineId,
  Settings,
  SystemStatus,
  Task,
  TaskId,
} from "../lib/protocol.gen";
import { viewTransition } from "../ui/motion";
import { loadSeen, loadSeenSince, saveSeen } from "./seen";

/** A panel beside a bot's chat (spec 15.1). */
export type BotPanel = "details" | "files" | "browser" | "screens";

export interface AppState {
  connection: ConnectionState;
  /** True once crews and bots arrived from the current connection. */
  loaded: boolean;
  loadError: string | null;
  system: SystemStatus | null;
  /** The daemon's part of Settings (spec 11.2); null until read, or from an older daemon. */
  settings: Settings | null;
  crews: Record<CrewId, Crew>;
  bots: Record<BotId, Bot>;
  /**
   * Each bot's conversation-list line (spec 8.3). Kept apart from the bot:
   * it changes with every chat item, and a new bot object re-renders
   * everything that shows the bot.
   */
  activity: Record<BotId, Activity | null>;
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
  /**
   * When the owner last looked at each bot's chat (`seen.ts`), for unread
   * replies and failed routine runs.
   */
  seenAt: Record<BotId, number>;
  /** Replies before this count as seen (`loadSeenSince`). */
  seenSince: number;
  /** When each bot last finished a reply (`Bot.lastReplyAt`), then every new one. */
  replyAt: Record<BotId, number>;
  /**
   * The panel the owner left beside each bot's chat (null: closed), so it is
   * back when they come back to the bot. Kept until the app closes (spec 15.1).
   */
  panels: Record<BotId, BotPanel | null>;
  selectedCrewId: CrewId | null;
  selectedBotId: BotId | null;
  selectCrew(crewId: CrewId | null): void;
  selectBot(botId: BotId): void;
  setPanel(botId: BotId, panel: BotPanel | null): void;
  /** The owner is looking at the bot's chat: what it said so far is seen. */
  markSeen(botId: BotId): void;
  /** Applies a record a call returned, before its notification arrives. */
  putCrew(crew: Crew): void;
  putBot(bot: Bot): void;
  /** Applies a delete a call returned, before its notification arrives. */
  dropCrew(crewId: CrewId): void;
  dropBot(botId: BotId): void;
  putDelivery(delivery: Delivery): void;
  putRoutine(routine: Routine): void;
  putBrowser(browser: BrowserState): void;
  putSettings(settings: Settings | null): void;
}

export type AppStore = StoreApi<AppState>;

const byCreation = <T extends { createdAt: number; id: string }>(a: T, b: T) =>
  a.createdAt - b.createdAt || a.id.localeCompare(b.id);

export function crewList(state: AppState): Crew[] {
  return Object.values(state.crews).sort(byCreation);
}

/** The bot's conversation-list line; see [`AppState.activity`]. */
export function activityOf(state: AppState, botId: BotId): Activity | null {
  return state.activity[botId] ?? null;
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
    settings: null,
    crews: {},
    bots: {},
    activity: {},
    deliveries: {},
    tasks: {},
    routines: {},
    browsers: {},
    seenAt: loadSeen(),
    seenSince: loadSeenSince(),
    replyAt: {},
    panels: {},
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
        get().markSeen(botId);
        viewTransition(() => set({ selectedCrewId: bot.crewId, selectedBotId: botId }));
      }
    },
    markSeen: (botId) => {
      const { seenAt, replyAt } = get();
      const next = { ...seenAt, [botId]: Math.max(Date.now(), replyAt[botId] ?? 0) };
      saveSeen(next);
      set({ seenAt: next });
    },
    setPanel: (botId, panel) => set((state) => ({ panels: { ...state.panels, [botId]: panel } })),
    putCrew: (crew) => set((state) => withCrew(state, crew)),
    putBot: (bot) => set((state) => withBot(state, bot)),
    dropCrew: (crewId) => set((state) => withoutCrew(state, crewId)),
    dropBot: (botId) => set((state) => withoutBots(state, [botId])),
    putRoutine: (routine) => set((state) => withRoutine(state, routine)),
    putBrowser: (browser) => set((state) => withBrowser(state, browser)),
    putSettings: (settings) => set({ settings }),
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
    return {
      bots: { ...state.bots, [bot.id]: bot },
      activity: { ...state.activity, [bot.id]: bot.lastActivity },
      replyAt: withReply(state.replyAt, bot.id, bot.lastReplyAt),
    };
  }
  const { [bot.id]: _gone, ...bots } = state.bots;
  return { bots, selectedBotId: state.selectedBotId === bot.id ? null : state.selectedBotId };
}

/** Without the entries of `record` that fail `keep`. */
function kept<T>(record: Record<string, T>, keep: (value: T) => boolean): Record<string, T> {
  return Object.fromEntries(Object.entries(record).filter(([, value]) => keep(value)));
}

/**
 * Deleted bots leave with what was theirs (spec 7.6): the daemon sends no
 * notification for each routine, task and delivery that went with them.
 */
function withoutBots(state: AppState, botIds: BotId[]): Partial<AppState> {
  const gone = new Set(botIds);
  return {
    bots: kept(state.bots, (bot) => !gone.has(bot.id)),
    activity: Object.fromEntries(Object.entries(state.activity).filter(([id]) => !gone.has(id))),
    replyAt: Object.fromEntries(Object.entries(state.replyAt).filter(([id]) => !gone.has(id))),
    routines: kept(state.routines, (routine) => !gone.has(routine.botId)),
    browsers: kept(state.browsers, (browser) => !gone.has(browser.botId)),
    deliveries: kept(state.deliveries, (delivery) => !gone.has(delivery.botId)),
    tasks: kept(
      state.tasks,
      (task) => !gone.has(task.requesterBotId) && !gone.has(task.assigneeBotId),
    ),
    selectedBotId:
      state.selectedBotId !== null && gone.has(state.selectedBotId) ? null : state.selectedBotId,
  };
}

function withoutCrew(state: AppState, crewId: CrewId): Partial<AppState> {
  const bots = Object.values(state.bots).filter((bot) => bot.crewId === crewId);
  const selected = state.selectedCrewId === crewId;
  return {
    ...withoutBots(
      state,
      bots.map((bot) => bot.id),
    ),
    crews: kept(state.crews, (crew) => crew.id !== crewId),
    selectedCrewId: selected ? null : state.selectedCrewId,
    selectedBotId: selected ? null : state.selectedBotId,
  };
}

export function applyEvent(state: AppState, event: ServerEvent): Partial<AppState> | null {
  switch (event.name) {
    case "crew.changed":
      return withCrew(state, event.params);
    case "crew.deleted":
      return withoutCrew(state, event.params.crewId);
    case "bot.changed":
      return withBot(state, event.params);
    case "bot.deleted":
      return withoutBots(state, [event.params.botId]);
    case "bot.state": {
      const bot = state.bots[event.params.botId];
      if (!bot) {
        return null;
      }
      const { state: botState, generation } = event.params;
      return { bots: { ...state.bots, [bot.id]: { ...bot, state: botState, generation } } };
    }
    case "bot.context": {
      const bot = state.bots[event.params.botId];
      if (!bot) {
        return null;
      }
      return { bots: { ...state.bots, [bot.id]: { ...bot, context: event.params.context } } };
    }
    case "chat.item": {
      const { item, activity } = event.params;
      if (!state.bots[item.botId] || !activity) {
        return null;
      }
      const reply = item.body.kind === "reply" ? item.updatedAt : null;
      return {
        activity: { ...state.activity, [item.botId]: activity },
        replyAt: withReply(state.replyAt, item.botId, reply),
      };
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

/** `replyAt` with a reply of `botId` at `at`, if it is newer. */
export function withReply(
  replyAt: Record<BotId, number>,
  botId: BotId,
  at: number | null,
): Record<BotId, number> {
  return at !== null && at > (replyAt[botId] ?? 0) ? { ...replyAt, [botId]: at } : replyAt;
}

/** The routines of `botId`, oldest first. */
export function routinesOf(state: AppState, botId: BotId): Routine[] {
  return Object.values(state.routines)
    .filter((routine) => routine.botId === botId)
    .sort(byCreation);
}
