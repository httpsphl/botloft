// The fake daemon's `bots.*` methods, following the daemon's rules for
// handles, colors, archiving, unarchiving, deleting and the notifications each change
// sends.

import type { FakeBotloft, Handlers } from "./fake";
import { purgeBot } from "./fakeDelete";
import { checkName, conflict, invalid, notFound } from "./fakeRules";
import { AVATAR_PALETTE, type Bot } from "./protocol.gen";

type BotMethods = Extract<keyof Handlers, `bots.${string}`>;

export function botHandlers(fake: FakeBotloft): Pick<Handlers, BotMethods> {
  return {
    "bots.list": ({ crewId }) => {
      if (crewId !== undefined) {
        fake.crew(crewId);
      }
      return fake.activeBots(crewId);
    },
    "bots.create": ({ crewId, name, role, instructions, color, model, agent }) => {
      const crew = fake.crew(crewId);
      const checked = checkName(name);
      const handle = fake.handle(checked, crewId);
      const index = [...fake.bots.values()].filter((bot) => bot.crewId === crewId).length;
      const bot: Bot = {
        id: fake.id("bot"),
        crewId,
        name: checked,
        handle,
        slug: handle,
        role: role.trim(),
        instructions,
        color: color ?? AVATAR_PALETTE[index % AVATAR_PALETTE.length] ?? "#FF7A59",
        paused: false,
        permissionMode: crew.autoMode ? "auto" : "default",
        agent: agent ?? "claude",
        agentModel: null,
        allowedCommands: [],
        planFirst: false,
        model: model ?? "default",
        modelInUse: null,
        effort: "default",
        effortDefault: null,
        context: null,
        background: [],
        connectors: [],
        state: crew.paused ? "offline" : "launching",
        generation: crew.paused ? null : 1,
        workspace: `C:\\Users\\owner\\Botloft\\${crew.slug}\\${handle}`,
        lastActivity: null,
        lastReplyAt: null,
        readThrough: null,
        createdAt: fake.now,
        archivedAt: null,
      };
      fake.bots.set(bot.id, bot);
      return fake.changedBot(bot);
    },
    "bots.update": ({ botId, name, role, instructions, color, allowedCommands, planFirst }) => {
      const bot = fake.bot(botId);
      if (name !== undefined) {
        bot.name = checkName(name);
        bot.handle = fake.handle(bot.name, bot.crewId, botId);
      }
      bot.role = role?.trim() ?? bot.role;
      bot.instructions = instructions ?? bot.instructions;
      bot.color = color ?? bot.color;
      bot.planFirst = planFirst ?? bot.planFirst;
      if (allowedCommands !== undefined) {
        bot.allowedCommands = [
          ...new Set(allowedCommands.map((line) => line.trim()).filter(Boolean)),
        ];
      }
      return fake.changedBot(bot);
    },
    "bots.setPaused": ({ botId, paused }) => {
      const bot = fake.bot(botId);
      bot.paused = paused;
      return fake.changedBot(bot);
    },
    "bots.setPermissionMode": ({ botId, mode }) => {
      const bot = fake.bot(botId);
      bot.permissionMode = mode;
      return fake.changedBot(bot);
    },
    "bots.setModel": ({ botId, model }) => {
      const bot = fake.bot(botId);
      if (bot.model !== model) {
        bot.model = model;
        // What a model does by itself is asked again of the new one.
        bot.effortDefault = null;
      }
      return fake.changedBot(bot);
    },
    "bots.setAgentModel": ({ botId, model }) => {
      const bot = fake.bot(botId);
      bot.agentModel = model;
      return fake.changedBot(bot);
    },
    "bots.setEffort": ({ botId, effort }) => {
      const bot = fake.bot(botId);
      bot.effort = effort;
      return fake.changedBot(bot);
    },
    "bots.compact": ({ botId }) => {
      const bot = fake.bot(botId);
      // A sleeping bot wakes up first (spec 7.7).
      const asleep = bot.state === "sleeping" && !bot.paused;
      if (
        !asleep &&
        bot.state !== "idle" &&
        bot.state !== "busy" &&
        bot.state !== "needs_approval"
      ) {
        throw conflict(`bot ${botId} is not running`);
      }
      // It waits behind the turn in progress; a test ends it with `setContext`.
      if (bot.context && !bot.context.compacting) {
        fake.setContext(botId, { ...bot.context, compacting: true });
      }
      return bot;
    },
    "bots.interrupt": ({ botId }) => {
      const bot = fake.bot(botId);
      if (bot.agent !== "claude") {
        throw invalid(`bots that run on ${bot.agent} cannot be stopped in the middle of a turn`);
      }
      if (bot.state === "busy" || bot.state === "needs_approval") {
        // A tool that waits for the owner is refused, and Claude Code ends
        // the turn as stopped.
        for (const approval of fake.chat.approvals.values()) {
          if (approval.botId === botId && approval.status === "pending") {
            void fake.call("approvals.answer", { approvalId: approval.id, allow: false });
          }
        }
        fake.chat.stopped(botId);
        fake.setBotState(botId, "idle");
      }
      return bot;
    },
    "bots.stopTask": ({ botId, taskId }) => {
      const bot = fake.bot(botId);
      if (!bot.background.some((task) => task.taskId === taskId)) {
        throw notFound(`task ${taskId} of bot ${botId}`);
      }
      // Claude Code ends it and sends the list without it.
      fake.setBackground(
        botId,
        bot.background.filter((task) => task.taskId !== taskId),
      );
      return null;
    },
    "bots.setConnectors": ({ botId, names }) => {
      const bot = fake.bot(botId);
      bot.connectors = [...new Set(names)];
      return fake.changedBot(bot);
    },
    "bots.restart": ({ botId, fresh }) => {
      const bot = fake.bot(botId);
      if (fresh) {
        // The conversation is forgotten: nothing to count until it tells.
        bot.context = null;
        fake.chat.add(botId, {
          kind: "notice",
          level: "info",
          code: "fresh_start",
          text: "A new conversation started here: what came before is not remembered.",
        });
        // A sleeping bot keeps sleeping (spec 7.7).
        if (bot.state === "sleeping") {
          return fake.changedBot(bot);
        }
      }
      fake.setBotState(botId, "launching", (bot.generation ?? 0) + 1);
      return bot;
    },
    "bots.archive": ({ botId }) => {
      const bot = fake.bot(botId, false);
      if (bot.archivedAt === null) {
        bot.archivedAt = fake.now;
        bot.state = "archived";
        fake.changedBot(bot);
        // An archived chief leaves its crew without one.
        const crew = fake.crews.get(bot.crewId);
        if (crew?.leadBotId === botId) {
          crew.leadBotId = null;
          fake.changedCrew(crew);
        }
      }
      return bot;
    },
    "bots.unarchive": ({ botId }) => {
      const bot = fake.bot(botId, false);
      if (bot.archivedAt !== null) {
        if (fake.crew(bot.crewId, false).archivedAt !== null) {
          throw conflict(`crew ${bot.crewId} is archived; unarchive the crew`);
        }
        if (fake.activeBots(bot.crewId).some((other) => other.handle === bot.handle)) {
          throw conflict(`another bot in this crew already answers to @${bot.handle}`);
        }
        bot.archivedAt = null;
        bot.state = "idle";
        fake.changedBot(bot);
      }
      return bot;
    },
    "bots.delete": ({ botId, recycleFolder }) => {
      const bot = fake.bot(botId, false);
      purgeBot(fake, bot);
      // A deleted chief leaves its crew without one.
      const crew = fake.crews.get(bot.crewId);
      if (crew?.leadBotId === botId) {
        crew.leadBotId = null;
        fake.changedCrew(crew);
      }
      const deleted = { botId, crewId: bot.crewId };
      fake.emit({ name: "bot.deleted", params: deleted });
      if (recycleFolder) {
        fake.recycle(bot.workspace);
      }
      return deleted;
    },
  };
}
