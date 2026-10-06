// The fake daemon's bot catalog (spec 26): the same twelve ids and
// categories as the daemon's sheets, with short stand-in texts. Adding a
// role goes through `bots.create`, so handles and notifications match.

import type { FakeBotloft, Handlers } from "./fake";
import { botHandlers } from "./fakeBots";
import { notFound } from "./fakeRules";
import type { BotEffort, BotTemplateCategory, BotTemplateFull } from "./protocol.gen";

type CatalogMethods = Extract<keyof Handlers, `catalog.${string}`>;

const ROLES: [id: string, category: BotTemplateCategory, name: string, effort?: BotEffort][] = [
  ["developer", "code", "Developer"],
  ["code-reviewer", "code", "Code Reviewer", "high"],
  ["qa-tester", "code", "QA Tester"],
  ["designer", "design", "Designer"],
  ["writer", "content", "Writer"],
  ["social-media", "content", "Social Media"],
  ["translator", "content", "Translator"],
  ["researcher", "research", "Researcher"],
  ["data-analyst", "research", "Data Analyst"],
  ["sales-prospector", "business", "Sales Prospector"],
  ["customer-support", "business", "Customer Support"],
  ["personal-assistant", "business", "Personal Assistant"],
];

/** Every role, as `catalog.get` returns it. */
export const FAKE_CATALOG: BotTemplateFull[] = ROLES.map(([id, category, name, effort]) => ({
  id,
  category,
  name,
  role: `${name} role`,
  summary: `What the ${name.toLowerCase()} does`,
  model: "default",
  effort: effort ?? "default",
  instructions: `You are a ${name.toLowerCase()} on this crew.\n\n### What you do\nThe ${name.toLowerCase()}'s work.`,
}));

function find(id: string): BotTemplateFull {
  const found = FAKE_CATALOG.find((role) => role.id === id);
  if (!found) {
    throw notFound(`bot template ${id}`);
  }
  return found;
}

export function catalogHandlers(fake: FakeBotloft): Pick<Handlers, CatalogMethods> {
  return {
    "catalog.list": ({ category }) =>
      FAKE_CATALOG.filter((role) => category === undefined || role.category === category).map(
        ({ id, category, name, role, summary }) => ({ id, category, name, role, summary }),
      ),
    "catalog.get": ({ id }) => find(id),
    "catalog.add": ({ crewId, templateId, name, role }) => {
      const template = find(templateId);
      const created = botHandlers(fake)["bots.create"]({
        crewId,
        name,
        role,
        instructions: template.instructions,
        model: template.model,
      });
      const bot = fake.bot(created.id);
      bot.effort = template.effort;
      return fake.changedBot(bot);
    },
  };
}
