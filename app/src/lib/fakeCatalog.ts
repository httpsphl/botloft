// The fake daemon's bot catalog (spec 26): the same ids and
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
  ["product-manager", "product", "Product Manager"],
  ["project-manager", "product", "Project Manager"],
  ["business-analyst", "product", "Business Analyst"],
  ["ux-researcher", "product", "UX Researcher"],
  ["agile-facilitator", "product", "Agile Facilitator"],
  ["goals-coach", "product", "Goals Coach"],
  ["meeting-secretary", "product", "Meeting Secretary"],
  ["process-analyst", "product", "Process Analyst"],
  ["operations-manager", "product", "Operations Manager"],
  ["recruiter", "product", "Recruiter"],
  ["onboarding-coach", "product", "Onboarding Coach"],
  ["customer-success", "product", "Customer Success"],
  ["event-planner", "product", "Event Planner"],
  ["travel-planner", "product", "Travel Planner"],
  ["seo-specialist", "marketing", "SEO Specialist"],
  ["email-marketer", "marketing", "Email Marketer"],
  ["content-strategist", "marketing", "Content Strategist"],
  ["copywriter", "marketing", "Copywriter"],
  ["ads-manager", "marketing", "Ads Manager"],
  ["video-scriptwriter", "marketing", "Video Scriptwriter"],
  ["newsletter-editor", "marketing", "Newsletter Editor"],
  ["community-manager", "marketing", "Community Manager"],
  ["ecommerce-manager", "marketing", "E-commerce Manager"],
  ["brand-strategist", "marketing", "Brand Strategist"],
  ["competitor-analyst", "marketing", "Competitor Analyst"],
  ["growth-marketer", "marketing", "Growth Marketer"],
  ["conversion-optimizer", "marketing", "Conversion Optimizer"],
  ["pr-writer", "marketing", "PR Writer"],
  ["reputation-manager", "marketing", "Reputation Manager"],
  ["proposal-writer", "marketing", "Proposal Writer"],
  ["sales-coach", "marketing", "Sales Coach"],
  ["pricing-analyst", "marketing", "Pricing Analyst"],
  ["software-architect", "code", "Software Architect"],
  ["frontend-developer", "code", "Frontend Developer"],
  ["backend-developer", "code", "Backend Developer"],
  ["mobile-developer", "code", "Mobile Developer"],
  ["database-engineer", "code", "Database Engineer"],
  ["devops-engineer", "code", "DevOps Engineer"],
  ["security-reviewer", "code", "Security Reviewer", "high"],
  ["data-engineer", "code", "Data Engineer"],
  ["editor", "content", "Editor"],
  ["ghostwriter", "content", "Ghostwriter"],
  ["fact-checker", "research", "Fact Checker"],
  ["academic-researcher", "research", "Academic Researcher"],
  ["tutor", "learning", "Tutor"],
  ["language-teacher", "learning", "Language Teacher"],
  ["brand-guardian", "design", "Brand Guardian"],
  ["ui-designer", "design", "UI Designer"],
  ["ux-architect", "design", "UX Architect"],
  ["accessibility-reviewer", "design", "Accessibility Reviewer"],
  ["presentation-designer", "design", "Presentation Designer"],
  ["infographic-designer", "design", "Infographic Designer"],
  ["image-prompt-writer", "design", "Image Prompt Writer"],
  ["design-critic", "design", "Design Critic"],
  ["bookkeeper", "finance", "Bookkeeper"],
  ["budget-planner", "finance", "Budget Planner"],
  ["financial-analyst", "finance", "Financial Analyst"],
  ["cash-flow-forecaster", "finance", "Cash Flow Forecaster"],
  ["bills-assistant", "finance", "Bills Assistant"],
  ["invoicing-assistant", "finance", "Invoicing Assistant"],
  ["tax-organizer", "finance", "Tax Organizer"],
  ["ppc-strategist", "marketing", "Search Ads Strategist"],
  ["paid-social-strategist", "marketing", "Paid Social Strategist"],
  ["ad-creative-strategist", "marketing", "Ad Creative Strategist"],
  ["ad-auditor", "marketing", "Ad Account Auditor"],
  ["tracking-specialist", "marketing", "Tracking Specialist"],
  ["search-query-analyst", "marketing", "Search Query Analyst"],
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
