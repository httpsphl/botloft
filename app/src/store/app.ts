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
  ChatItemId,
  Crew,
  CrewId,
  Delivery,
  MessageId,
  Question,
  QuestionId,
  Routine,
  RoutineId,
  Settings,
  SystemStatus,
  Task,
  TaskId,
} from "../lib/protocol.gen";
import { type Call, withCall } from "./calls";
import { withQuestion } from "./questions";
import { loadMarked, loadSeen, loadSeenSince, seenActions, withReply } from "./seen";

/** A panel beside a bot's chat (spec 15.1). */
export type BotPanel = "details" | "files" | "browser" | "screens" | "terminal";

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
  /** Messages bots sent each other since the app connected (`calls.ts`). */
  calls: Record<MessageId, Call>;
  tasks: Record<TaskId, Task>;
  /** Every routine that is not archived (spec 20), then every change. */
  routines: Record<RoutineId, Routine>;
  /** The bots' browsers that are not closed (spec 21.7), then every change. */
  browsers: Record<BotId, BrowserState>;
  /** Questions waiting for the owner (spec 23.5), then every change. */
  questions: Record<QuestionId, Question>;
  /** A page open in the middle instead of a crew or a bot. */
  page: Page;
  /** The chat item a search result opened, to show it (spec 8.8). */
  focus: { botId: BotId; itemId: ChatItemId } | null;
  /**
   * When the owner last looked at each bot's chat (`seen.ts`), for unread
   * replies and failed routine runs.
   */
  seenAt: Record<BotId, number>;
  /** Replies before this count as seen (`loadSeenSince`). */
  seenSince: number;
  /** When each bot last finished a reply (`Bot.lastReplyAt`), then every new one. */
  replyAt: Record<BotId, number>;
  /** Bots the owner marked unread themselves, until they open them again. */
  markedUnread: Record<BotId, true>;
  /**
   * The panel the owner left beside each bot's chat (null: closed), so it is
   * back when they come back to the bot. Kept until the app closes (spec 15.1).
   */
  panels: Record<BotId, BotPanel | null>;
  selectedCrewId: CrewId | null;
  selectedBotId: BotId | null;
  selectCrew(crewId: CrewId | null): void;
  selectBot(botId: BotId): void;
  openPage(page: Exclude<Page, null>): void;
  /** Opens the bot's chat at one of its items. */
  openAt(botId: BotId, itemId: ChatItemId): void;
  setPanel(botId: BotId, panel: BotPanel | null): void;
  /** The owner is looking at the bot's chat: what it said so far is seen. */
  markSeen(botId: BotId): void;
  /** Marks the bot unread until its chat is opened; leaves it if it is open. */
  markUnread(botId: BotId): void;
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

/** The question box (spec 23.6), the search (spec 8.8) or every routine (spec 20.9). */
export type Page = "questions" | "search" | "routines" | null;

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
  return createStore<AppState>()((set, get) => {
    /** Opens a bot's chat; with `focus`, at that item, even if it is open. */
    const openBot = (botId: BotId, focus: AppState["focus"]) => {
      const bot = get().bots[botId];
      if (!bot) {
        return;
      }
      const change = { selectedCrewId: bot.crewId, selectedBotId: botId, page: null, focus };
      if (get().selectedBotId !== botId || get().page) {
        get().markSeen(botId);
        set(change);
      } else if (focus) {
        set({ focus });
      }
    };
    return {
      connection: api.connection(),
      loaded: false,
      loadError: null,
      system: null,
      settings: null,
      crews: {},
      bots: {},
      activity: {},
      deliveries: {},
      calls: {},
      tasks: {},
      routines: {},
      browsers: {},
      questions: {},
      page: null,
      focus: null,
      seenAt: loadSeen(),
      seenSince: loadSeenSince(),
      replyAt: {},
      markedUnread: loadMarked(),
      panels: {},
      selectedCrewId: null,
      selectedBotId: null,
      selectCrew: (crewId) => {
        const { selectedCrewId, selectedBotId, page } = get();
        if (selectedCrewId !== crewId || selectedBotId !== null || page) {
          set({ selectedCrewId: crewId, selectedBotId: null, page: null, focus: null });
        }
      },
      selectBot: (botId) => openBot(botId, null),
      openAt: (botId, itemId) => openBot(botId, { botId, itemId }),
      openPage: (page) => {
        if (get().page !== page) {
          set({ page, selectedCrewId: null, selectedBotId: null });
        }
      },
      ...seenActions(get, set),
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
    };
  });
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
    questions: kept(state.questions, (question) => !gone.has(question.botId)),
    deliveries: kept(state.deliveries, (delivery) => !gone.has(delivery.botId)),
    calls: kept(state.calls, (call) => !gone.has(call.fromBotId) && !gone.has(call.toBotId)),
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
    case "message.created":
      return withCall(state, event.params);
    case "delivery.changed":
      return { deliveries: { ...state.deliveries, [event.params.messageId]: event.params } };
    case "task.changed":
      return { tasks: { ...state.tasks, [event.params.id]: event.params } };
    case "routine.changed":
      return withRoutine(state, event.params);
    case "browser.changed":
      return withBrowser(state, event.params);
    case "question.changed":
      return withQuestion(state, event.params);
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
    // Closed is no browser, unless it is open in a window of its own.
    browsers:
      browser.status === "closed" && !browser.window ? rest : { ...rest, [browser.botId]: browser },
  };
}

/** Adds or replaces a routine; an archived one leaves the store. */
function withRoutine(state: AppState, routine: Routine): Partial<AppState> {
  const { [routine.id]: _, ...rest } = state.routines;
  return { routines: routine.archivedAt === null ? { ...rest, [routine.id]: routine } : rest };
}

/** The routines of `botId`, oldest first. */
export function routinesOf(state: AppState, botId: BotId): Routine[] {
  return Object.values(state.routines)
    .filter((routine) => routine.botId === botId)
    .sort(byCreation);
}
