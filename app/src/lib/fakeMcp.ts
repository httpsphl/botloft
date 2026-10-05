// The fake daemon's connected tools (spec 25): the servers the owner
// registered and the bots that use them. Secrets are not kept: like the
// daemon, the list never carries their values, only their names.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, invalid, notFound, slugify } from "./fakeRules";
import type { BotMcp, McpOverview, McpServer, McpServerId } from "./protocol.gen";

type McpMethods = Extract<keyof Handlers, `mcp.${string}` | "bot.mcp.set">;

export class FakeMcp {
  private servers: McpServer[] = [];
  private readonly uses = new Map<string, McpServerId[]>();

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

  handlers(): Pick<Handlers, McpMethods> {
    return {
      "mcp.servers": () => this.overview(),
      "mcp.save": (params) => {
        const name = params.name.trim();
        const slug = slugify(name, "").replace(/-/g, "_");
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
      },
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
