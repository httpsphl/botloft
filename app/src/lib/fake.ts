// FakeBotloft: an in-memory daemon for component tests (spec 15.1). It
// follows the daemon's rules closely enough for the UI: validation,
// handles, archiving, and the notifications each change sends.

import type { BotloftApi } from "./api";
import { botHandlers } from "./fakeBots";
import { FakeBrowser } from "./fakeBrowser";
import { FakeChat } from "./fakeChat";
import { FakeConversation } from "./fakeConversation";
import { crewHandlers } from "./fakeCrews";
import { FakeFiles } from "./fakeFiles";
import { FakeRoutines } from "./fakeRoutines";
import { conflict, invalid, notFound, slugify } from "./fakeRules";
import { FakeScreens } from "./fakeScreens";
import {
  type Bot,
  type BotId,
  type BotState,
  type Crew,
  type CrewId,
  PROTOCOL_VERSION,
  type Settings,
  type SystemStatus,
} from "./protocol.gen";
import type {
  ConnectionState,
  Method,
  Params,
  ParamsArg,
  Result,
  RpcError,
  ServerEvent,
} from "./rpc";

export type Handlers = { [M in Method]: (params: Params<M>) => Result<M> };

/** Shared by every fake, so ids never repeat between tests (caches key on them). */
let counter = 0;

export class FakeBotloft implements BotloftApi {
  readonly crews = new Map<CrewId, Crew>();
  readonly bots = new Map<BotId, Bot>();
  system: SystemStatus = {
    daemonVersion: "0.1.0",
    protocol: PROTOCOL_VERSION,
    uptimeMs: 1000,
    claudeVersion: "2.1.284",
    runtimeError: null,
    claudePath: "C:\\Users\\owner\\.local\\bin\\claude.exe",
    claudeSignedIn: true,
    account: {
      name: "Ana Lima",
      claude: { email: "ana@example.com", plan: "max", organization: null },
    },
    deliveries: { pending: 0, dead: 0 },
    usage: null,
  };
  /** How many times the app asked for a new Claude Code check. */
  refreshes = 0;
  settings: Settings = { startWithWindows: true, keepAwake: true, approvalWaitMinutes: 60 };
  /** Every call, in order. */
  readonly calls: { method: Method; params: unknown }[] = [];
  readonly chat = new FakeChat(this);
  readonly conversation = new FakeConversation(this);
  readonly routines = new FakeRoutines(this);
  readonly files = new FakeFiles(this);
  readonly browser = new FakeBrowser(this);
  readonly screens = new FakeScreens(this);
  /** Clock for created and updated times. */
  now = Date.now();
  closed = false;
  private state: ConnectionState = { kind: "open", daemonVersion: "0.1.0" };
  private readonly listeners = new Set<(event: ServerEvent) => void>();
  private readonly connectionListeners = new Set<(state: ConnectionState) => void>();
  private readonly failures = new Map<Method, RpcError>();

  call<M extends Method>(method: M, ...params: ParamsArg<M>): Promise<Result<M>> {
    this.calls.push({ method, params: params[0] });
    const failure = this.failures.get(method);
    if (failure) {
      this.failures.delete(method);
      return Promise.reject(failure);
    }
    try {
      const handler = this.handlers[method] as (params: unknown) => Result<M>;
      return Promise.resolve(structuredClone(handler(params[0])));
    } catch (error) {
      return Promise.reject(error);
    }
  }

  subscribe(listener: (event: ServerEvent) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  connection(): ConnectionState {
    return this.state;
  }

  onConnection(listener: (state: ConnectionState) => void): () => void {
    this.connectionListeners.add(listener);
    return () => this.connectionListeners.delete(listener);
  }

  /** Satisfies `Client`; records that the app let go of the connection. */
  close(): void {
    this.closed = true;
  }

  // Test helpers.

  /** The next call to `method` fails with `error`. */
  failNext(method: Method, error: RpcError): void {
    this.failures.set(method, error);
  }

  /** Sends a notification, unless disconnected: then it is lost, as on a socket. */
  emit(event: ServerEvent): void {
    if (this.state.kind !== "open") {
      return;
    }
    for (const listener of this.listeners) {
      listener(structuredClone(event));
    }
  }

  setConnection(state: ConnectionState): void {
    this.state = state;
    for (const listener of this.connectionListeners) {
      listener(state);
    }
  }

  addCrew(name: string): Crew {
    return this.handlers["crews.create"]({ name });
  }

  addBot(crewId: CrewId, name: string, role = ""): Bot {
    return this.handlers["bots.create"]({ crewId, name, role, instructions: "" });
  }

  setBotState(botId: BotId, state: BotState, generation?: number): void {
    const bot = this.bot(botId);
    bot.state = state;
    bot.generation = generation ?? bot.generation ?? 1;
    this.emit({ name: "bot.state", params: { botId, state, generation: bot.generation } });
  }

  /** A new id with `prefix`, like the daemon's. */
  id(prefix: string): string {
    counter += 1;
    return `${prefix}_${String(counter).padStart(4, "0")}`;
  }

  /** The crew; archived ones only when `active` is false. */
  crew(crewId: CrewId, active = true): Crew {
    const crew = this.crews.get(crewId);
    if (!crew) {
      throw notFound(`crew ${crewId}`);
    }
    if (active && crew.archivedAt !== null) {
      throw conflict(`crew ${crewId} is archived`);
    }
    return crew;
  }

  /** The bot; archived ones only when `active` is false. */
  bot(botId: BotId, active = true): Bot {
    const bot = this.bots.get(botId);
    if (!bot) {
      throw notFound(`bot ${botId}`);
    }
    if (active && bot.archivedAt !== null) {
      throw conflict(`bot ${botId} is archived`);
    }
    return bot;
  }

  activeBots(crewId?: CrewId): Bot[] {
    return [...this.bots.values()].filter(
      (bot) => bot.archivedAt === null && (crewId === undefined || bot.crewId === crewId),
    );
  }

  changedCrew(crew: Crew): Crew {
    this.emit({ name: "crew.changed", params: crew });
    return crew;
  }

  changedBot(bot: Bot): Bot {
    this.emit({ name: "bot.changed", params: bot });
    return bot;
  }

  handle(name: string, crewId: CrewId, except?: BotId): string {
    const handle = slugify(name, "bot");
    const taken = this.activeBots(crewId).some((bot) => bot.handle === handle && bot.id !== except);
    if (taken) {
      throw invalid(`@${handle} is already used in this crew`);
    }
    return handle;
  }

  private readonly handlers: Handlers = {
    "session.hello": () => ({
      daemonVersion: this.system.daemonVersion,
      protocol: PROTOCOL_VERSION,
    }),
    "system.status": () => this.system,
    "system.refresh": () => {
      this.refreshes += 1;
      return this.system;
    },
    "settings.get": () => this.settings,
    "settings.update": (change) => {
      this.settings = {
        startWithWindows: change.startWithWindows ?? this.settings.startWithWindows,
        keepAwake: change.keepAwake ?? this.settings.keepAwake,
        approvalWaitMinutes: change.approvalWaitMinutes ?? this.settings.approvalWaitMinutes,
      };
      return this.settings;
    },
    ...crewHandlers(this),
    ...botHandlers(this),
    ...this.chat.handlers(),
    ...this.conversation.handlers(),
    ...this.files.handlers(),
    ...this.browser.handlers(),
    ...this.screens.handlers(),
    ...this.routines.handlers(),
  };
}
