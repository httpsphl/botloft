// The crew templates (spec 29.1): which catalog roles each starts with. What
// the owner reads about them is in `i18n/<language>/crewTemplates.ts`. A
// crew holds up to 12 bots, the chief among them, so a template stays well
// under that.

import type { Messages } from "../../i18n";

export type CrewTemplateId = keyof Messages["crewTemplates"]["items"];

export interface CrewTemplate {
  id: CrewTemplateId;
  /** Catalog role ids, in the order the bots are added. */
  roles: readonly string[];
}

export const CREW_TEMPLATES: readonly CrewTemplate[] = [
  {
    id: "content-studio",
    roles: ["content-strategist", "writer", "editor", "seo-specialist", "social-media", "designer"],
  },
  {
    id: "software-team",
    roles: [
      "software-architect",
      "frontend-developer",
      "backend-developer",
      "qa-tester",
      "code-reviewer",
      "technical-writer",
    ],
  },
  {
    id: "online-store",
    roles: [
      "ecommerce-manager",
      "copywriter",
      "customer-support",
      "returns-assistant",
      "ads-manager",
      "bookkeeper",
    ],
  },
  {
    id: "growth-team",
    roles: [
      "growth-marketer",
      "ppc-strategist",
      "paid-social-strategist",
      "ad-creative-strategist",
      "tracking-specialist",
      "email-marketer",
    ],
  },
  {
    id: "research-desk",
    roles: [
      "researcher",
      "academic-researcher",
      "fact-checker",
      "data-analyst",
      "editor",
      "writer",
    ],
  },
  {
    id: "back-office",
    roles: [
      "bookkeeper",
      "invoicing-assistant",
      "cash-flow-forecaster",
      "bills-assistant",
      "tax-organizer",
      "contract-reader",
    ],
  },
  {
    id: "personal-office",
    roles: [
      "personal-assistant",
      "travel-planner",
      "bills-assistant",
      "tax-organizer",
      "goals-coach",
      "meeting-secretary",
    ],
  },
  {
    id: "course-studio",
    roles: [
      "course-creator",
      "video-scriptwriter",
      "presentation-designer",
      "designer",
      "editor",
      "social-media",
    ],
  },
  {
    id: "game-studio",
    roles: [
      "game-designer",
      "game-narrative-designer",
      "level-designer",
      "game-economy-designer",
      "playtest-analyst",
      "designer",
    ],
  },
  {
    id: "people-team",
    roles: [
      "recruiter",
      "onboarding-coach",
      "training-designer",
      "hr-policy-writer",
      "engagement-survey-analyst",
      "performance-review-coach",
    ],
  },
];
