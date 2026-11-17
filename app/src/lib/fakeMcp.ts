// The fake daemon's connected tools (spec 25): the servers the owner
// registered and the bots that use them. Secrets are not kept: like the
// daemon, the list never carries their values, only their names.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, invalid, notFound, slugify } from "./fakeRules";
import type {
  BotMcp,
  ClaudeMcpServer,
  McpOverview,
  McpSaveParams,
  McpServer,
  McpServerId,
} from "./protocol.gen";

type McpMethods = Extract<keyof Handlers, `mcp.${string}` | "bot.mcp.set">;

const slugOf = (name: string) => slugify(name.trim(), "").replace(/-/g, "_");

export class FakeMcp {
  private servers: McpServer[] = [];
  private readonly uses = new Map<string, McpServerId[]>();
  /** What the owner's Claude Code has (spec 25.8); `null` when it has no file. */
  claudeCode: ClaudeMcpServer[] | null = null;

  constructor(private readonly fake: FakeBotloft) {}

  overview(): McpOverview {
    const bots: BotMcp[] = [...this.uses]
      .filter(([, serverIds]) => serverIds.length > 0)
      .map(([botId, serverIds]) => ({ botId, serverIds: [...serverIds], states: [] }));
    return { servers: structuredClone(this.servers), bots };
  }

  private changed(): McpOverview {
    const overview = this.overview();
    this.fake.emit({ name: "mcp.servers", params: overview });
    return overview;
  }

  private save(params: McpSaveParams): McpOverview {
    const name = params.name.trim();
    const slug = slugOf(name);
    if (!slug || slug === "botloft") {
      throw invalid("that name cannot be used for a tool");
    }
    if (params.kind === "http" && !/^https?:\/\//.test(params.url ?? "")) {
      throw invalid("url must start with http:// or https://");
    }
    if (params.kind === "stdio" && !params.command?.trim()) {
      throw invalid("command must not be empty");
    }
    const old = this.servers.find((server) => server.id === params.serverId);
    if (params.serverId && !old) {
      throw notFound(`tool ${params.serverId}`);
    }
    if (this.servers.some((server) => server.slug === slug && server.id !== old?.id)) {
      throw conflict("slug is already in use");
    }
    const server: McpServer = {
      id: old?.id ?? (this.fake.id("msv") as McpServerId),
      name,
      slug,
      kind: params.kind,
      url: params.kind === "http" ? (params.url ?? null) : null,
      command: params.kind === "stdio" ? (params.command ?? null) : null,
      args: params.args,
      headerNames: Object.keys(params.headers),
      envNames: Object.keys(params.env),
      description: params.description.trim(),
      createdAt: old?.createdAt ?? this.fake.now,
    };
    this.servers = old
      ? this.servers.map((each) => (each.id === old.id ? server : each))
      : [...this.servers, server];
    return this.changed();
  }

  handlers(): Pick<Handlers, McpMethods> {
    return {
      "mcp.servers": () => this.overview(),
      "mcp.fromClaude": () => {
        const slugs = new Set(this.servers.map((server) => server.slug));
        return {
          found: this.claudeCode !== null,
          servers: (this.claudeCode ?? []).map((each) => ({
            ...each,
            saved: slugs.has(slugOf(each.name)),
          })),
        };
      },
      "mcp.importClaude": ({ picks }) => {
        const saves: McpSaveParams[] = picks.map((pick) => {
          const each = this.claudeCode?.find(
            (server) => server.name === pick.name && server.folder === pick.folder,
          );
          if (!each) {
            throw notFound(`Claude Code tool ${pick.name}`);
          }
          if (!each.kind || each.problem) {
            throw invalid(`${pick.name} cannot be brought as it is`);
          }
          // The fake keeps no secrets: the values are left empty.
          const empty = (names: string[]) => Object.fromEntries(names.map((name) => [name, ""]));
          return {
            serverId: null,
            name: each.name,
            kind: each.kind,
            url: each.url,
            command: each.command,
            args: each.args,
            headers: empty(each.headerNames),
            env: empty(each.envNames),
            description: "",
          };
        });
        let overview = this.overview();
        for (const save of saves) {
          overview = this.save(save);
        }
        return overview;
      },
      "mcp.save": (params) => this.save(params),
      "mcp.delete": ({ serverId }) => {
        if (!this.servers.some((server) => server.id === serverId)) {
          throw notFound(`tool ${serverId}`);
        }
        this.servers = this.servers.filter((server) => server.id !== serverId);
        for (const [botId, ids] of this.uses) {
          this.uses.set(
            botId,
            ids.filter((id) => id !== serverId),
          );
        }
        return this.changed();
      },
      "bot.mcp.set": ({ botId, serverIds }) => {
        this.fake.bot(botId);
        for (const id of serverIds) {
          if (!this.servers.some((server) => server.id === id)) {
            throw notFound(`tool ${id}`);
          }
        }
        const unique = [...new Set(serverIds)];
        this.uses.set(botId, unique);
        const bot: BotMcp = { botId, serverIds: unique, states: [] };
        this.fake.emit({ name: "bot.mcp", params: bot });
        this.changed();
        return bot;
      },
    };
  }
}
