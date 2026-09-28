// FakeBotloft: an in-memory daemon for component tests (spec 15.1). It
// follows the daemon's rules closely enough for the UI: validation,
// handles, archiving, and the notifications each change sends.

import type { BotloftApi } from "./api";
import { conversationHandlers } from "./fakeConversation";
import { checkName, conflict, invalid, notFound, slugify } from "./fakeRules";
import { FakeTerminals } from "./fakeTerminal";
import {
  AVATAR_PALETTE,
  type Bot,
  type BotId,
  type BotState,
  type Crew,
  type CrewId,
  type Delivery,
  type Message,
  type SystemStatus,
  type Task,
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

export class FakeBotloft implements BotloftApi {
  readonly crews = new Map<CrewId, Crew>();
  readonly bots = new Map<BotId, Bot>();
  readonly messages: Message[] = [];
  readonly deliveries = new Map<string, Delivery>();
  readonly tasks = new Map<string, Task>();
  system: SystemStatus = {
    daemonVersion: "0.1.0",
    protocol: 1,
    uptimeMs: 1000,
    claudeVersion: "2.1.284",
    runtimeError: null,
    deliveries: { pending: 0, dead: 0 },
  };
  /** Every call, in order. */
  readonly calls: { method: Method; params: unknown }[] = [];
  readonly terminals = new FakeTerminals((event) => this.emit(event));
  now = 1_760_000_000_000;
  closed = false;
  private counter = 0;
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
    if (generation !== undefined && generation !== bot.generation) {
      this.terminals.restart(botId, generation);
    }
    bot.generation = generation ?? bot.generation ?? 1;
    this.emit({ name: "bot.state", params: { botId, state, generation: bot.generation } });
  }

  /** A new id with `prefix`, like the daemon's. */
  id(prefix: string): string {
    this.counter += 1;
    return `${prefix}_${String(this.counter).padStart(4, "0")}`;
  }

  private crew(crewId: CrewId, active = true): Crew {
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

  private activeBots(crewId?: CrewId): Bot[] {
    return [...this.bots.values()].filter(
      (bot) => bot.archivedAt === null && (crewId === undefined || bot.crewId === crewId),
    );
  }

  private changedCrew(crew: Crew): Crew {
    this.emit({ name: "crew.changed", params: crew });
    return crew;
  }

  private changedBot(bot: Bot): Bot {
    this.emit({ name: "bot.changed", params: bot });
    return bot;
  }

  private handle(name: string, crewId: CrewId, except?: BotId): string {
    const handle = slugify(name, "bot");
    const taken = this.activeBots(crewId).some((bot) => bot.handle === handle && bot.id !== except);
    if (taken) {
      throw invalid(`@${handle} is already used in this crew`);
    }
    return handle;
  }

  private readonly handlers: Handlers = {
    "session.hello": () => ({ daemonVersion: this.system.daemonVersion, protocol: 1 }),
    "system.status": () => this.system,
    "crews.list": () => [...this.crews.values()].filter((crew) => crew.archivedAt === null),
    "crews.create": ({ name }) => {
      const checked = checkName(name);
      const crew: Crew = {
        id: this.id("crw"),
        name: checked,
        slug: slugify(checked, "crew"),
        paused: false,
        createdAt: this.now,
        archivedAt: null,
      };
      this.crews.set(crew.id, crew);
      return this.changedCrew(crew);
    },
    "crews.rename": ({ crewId, name }) => {
      const crew = this.crew(crewId);
      crew.name = checkName(name);
      return this.changedCrew(crew);
    },
    "crews.setPaused": ({ crewId, paused }) => {
      const crew = this.crew(crewId);
      crew.paused = paused;
      return this.changedCrew(crew);
    },
    "crews.archive": ({ crewId }) => {
      const crew = this.crew(crewId, false);
      if (crew.archivedAt === null) {
        crew.archivedAt = this.now;
        for (const bot of this.activeBots(crewId)) {
          bot.archivedAt = this.now;
          bot.state = "archived";
          this.changedBot(bot);
        }
        this.changedCrew(crew);
      }
      return crew;
    },
    "bots.list": ({ crewId }) => {
      if (crewId !== undefined) {
        this.crew(crewId);
      }
      return this.activeBots(crewId);
    },
    "bots.create": ({ crewId, name, role, instructions, color }) => {
      const crew = this.crew(crewId);
      const checked = checkName(name);
      const handle = this.handle(checked, crewId);
      const index = [...this.bots.values()].filter((bot) => bot.crewId === crewId).length;
      const bot: Bot = {
        id: this.id("bot"),
        crewId,
        name: checked,
        handle,
        slug: handle,
        role: role.trim(),
        instructions,
        color: color ?? AVATAR_PALETTE[index % AVATAR_PALETTE.length] ?? "#FF7A59",
        paused: false,
        state: crew.paused ? "offline" : "launching",
        generation: crew.paused ? null : 1,
        workspace: `C:\\Users\\owner\\Botloft\\${crew.slug}\\${handle}`,
        createdAt: this.now,
        archivedAt: null,
      };
      this.bots.set(bot.id, bot);
      return this.changedBot(bot);
    },
    "bots.update": ({ botId, name, role, instructions, color }) => {
      const bot = this.bot(botId);
      if (name !== undefined) {
        bot.name = checkName(name);
        bot.handle = this.handle(bot.name, bot.crewId, botId);
      }
      bot.role = role?.trim() ?? bot.role;
      bot.instructions = instructions ?? bot.instructions;
      bot.color = color ?? bot.color;
      return this.changedBot(bot);
    },
    "bots.setPaused": ({ botId, paused }) => {
      const bot = this.bot(botId);
      bot.paused = paused;
      return this.changedBot(bot);
    },
    "bots.restart": ({ botId }) => {
      const bot = this.bot(botId);
      this.setBotState(botId, "launching", (bot.generation ?? 0) + 1);
      return bot;
    },
    "bots.archive": ({ botId }) => {
      const bot = this.bot(botId, false);
      if (bot.archivedAt === null) {
        bot.archivedAt = this.now;
        bot.state = "archived";
        this.changedBot(bot);
      }
      return bot;
    },
    ...conversationHandlers(this),
  };
}
