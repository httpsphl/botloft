// The fake daemon's `bots.*` methods, following the daemon's rules for
// handles, colors, archiving and the notifications each change sends.

import type { FakeBotloft, Handlers } from "./fake";
import { checkName } from "./fakeRules";
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
    "bots.create": ({ crewId, name, role, instructions, color, model }) => {
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
        permissionMode: "default",
        model: model ?? "default",
        modelInUse: null,
        state: crew.paused ? "offline" : "launching",
        generation: crew.paused ? null : 1,
        workspace: `C:\\Users\\owner\\Botloft\\${crew.slug}\\${handle}`,
        lastActivity: null,
        createdAt: fake.now,
        archivedAt: null,
      };
      fake.bots.set(bot.id, bot);
      return fake.changedBot(bot);
    },
    "bots.update": ({ botId, name, role, instructions, color }) => {
      const bot = fake.bot(botId);
      if (name !== undefined) {
        bot.name = checkName(name);
        bot.handle = fake.handle(bot.name, bot.crewId, botId);
      }
      bot.role = role?.trim() ?? bot.role;
      bot.instructions = instructions ?? bot.instructions;
      bot.color = color ?? bot.color;
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
      bot.model = model;
      return fake.changedBot(bot);
    },
    "bots.restart": ({ botId }) => {
      const bot = fake.bot(botId);
      fake.setBotState(botId, "launching", (bot.generation ?? 0) + 1);
      return bot;
    },
    "bots.archive": ({ botId }) => {
      const bot = fake.bot(botId, false);
      if (bot.archivedAt === null) {
        bot.archivedAt = fake.now;
        bot.state = "archived";
        fake.changedBot(bot);
      }
      return bot;
    },
  };
}
