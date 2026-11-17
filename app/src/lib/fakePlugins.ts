// The fake daemon's side of the owner's Claude Code plugins (spec 41): the
// plugins on in the owner's Claude Code and which agents may use each one,
// chosen one by one or for every agent.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, notFound } from "./fakeRules";
import type { BotId, Plugin, PluginList } from "./protocol.gen";

export class FakePlugins {
  /** Each with the agents chosen one by one in `botIds`. */
  readonly plugins: Plugin[] = [];
  /** Every plugin for every agent on Claude Code. */
  everyone = false;

  constructor(private readonly fake: FakeBotloft) {}

  /** A plugin on in the owner's Claude Code, as the daemon would find it. */
  add(plugin: Partial<Plugin> & { name: string }): Plugin {
    const made: Plugin = {
      description: "",
      version: null,
      source: "claude_code",
      skills: [],
      agents: [],
      hooks: [],
      languageServers: [],
      mcpServers: [],
      programs: false,
      needsSettings: false,
      botIds: [],
      everyone: false,
      ...plugin,
    };
    this.plugins.push(made);
    this.plugins.sort((a, b) => a.name.localeCompare(b.name));
    return made;
  }

  /** The plugin as the daemon answers it: the agents that use it. */
  private shown(plugin: Plugin): Plugin {
    const claude = [...this.fake.bots.values()]
      .filter((bot) => bot.archivedAt === null && bot.agent === "claude")
      .map((bot) => bot.id);
    const botIds =
      this.everyone || plugin.everyone ? claude : plugin.botIds.filter((id) => claude.includes(id));
    return { ...plugin, botIds };
  }

  private list(): PluginList {
    return { plugins: this.plugins.map((plugin) => this.shown(plugin)), everyone: this.everyone };
  }

  private find(name: string): Plugin {
    const plugin = this.plugins.find((have) => have.name === name);
    if (!plugin) {
      throw notFound(`plugin ${name}`);
    }
    return plugin;
  }

  handlers(): Pick<Handlers, "plugins.list" | "plugins.share" | "plugins.everyone"> {
    return {
      "plugins.list": () => this.list(),
      "plugins.share": ({ name, botIds }) => {
        const plugin = this.find(name);
        const wanted: BotId[] = [];
        for (const botId of botIds) {
          if (this.fake.bot(botId).agent !== "claude") {
            throw conflict("only agents on Claude Code can use plugins");
          }
          if (!wanted.includes(botId)) {
            wanted.push(botId);
          }
        }
        plugin.botIds = wanted;
        return this.shown(plugin);
      },
      "plugins.everyone": ({ name, on }) => {
        if (name === undefined) {
          this.everyone = on;
        } else {
          this.find(name).everyone = on;
        }
        return this.list();
      },
    };
  }
}
