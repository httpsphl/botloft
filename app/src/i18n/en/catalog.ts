import { base } from "./catalogBase";
import { contentAndLearning } from "./catalogContent";
import { design } from "./catalogDesign";
import { engineering } from "./catalogEngineering";
import { engineeringMore } from "./catalogEngineeringMore";
import { finance } from "./catalogFinance";
import { games } from "./catalogGames";
import { health } from "./catalogHealth";
import { legal } from "./catalogLegal";
import { marketing } from "./catalogMarketing";
import { paidMedia } from "./catalogPaidMedia";
import { people } from "./catalogPeople";
import { product } from "./catalogProduct";
import { sectors } from "./catalogSectors";
import { security } from "./catalogSecurity";

// The Bot agency (spec 26): ready-made bots the owner adds to a crew. The
// roles are the daemon's sheets, by id; this is what the owner reads about
// each. Plain words: "bot" and "crew", never "template" or "prompt".

export const catalog = {
  title: "Agent agency",
  open: "Agent agency",
  openHint: "Ready-made agents you can add to this crew",
  intro:
    "Pick an agent for the job. It joins this crew ready to work, and you can change anything about it afterwards.",
  invite: {
    title: "Who do you want on your crew?",
    body: "Not sure what to create? Pick an agent for the job. You can change anything about it afterwards.",
  },
  search: "Search agents",
  filter: "Kinds of agents",
  categories: {
    all: "All",
    code: "Code",
    design: "Design",
    content: "Content",
    research: "Research",
    business: "Business",
    product: "Product & management",
    marketing: "Marketing & sales",
    learning: "Learning",
    finance: "Finance",
    people: "People & HR",
    security: "Security",
    health: "Health",
    games: "Games",
    legal: "Legal",
  },
  none: "No agent matches that.",
  loading: "Loading the agents…",
  add: "Add to crew",
  learnMore: "Learn more",
  back: "Back to all agents",
  seeAll: "See all agents",
  joined: (name: string) => `${name} joined the crew`,
  customize: "Customize",
  failed: {
    load: "Could not load the agents",
    add: "Could not add the agent",
  },
  detail: {
    what: "What it does",
    when: "When to call it",
    pairs: "Works well with",
    technical: "What the agent is told when it starts",
  },
  roles: {
    ...base,
    ...product,
    ...marketing,
    ...engineering,
    ...contentAndLearning,
    ...design,
    ...finance,
    ...paidMedia,
    ...people,
    ...engineeringMore,
    ...security,
    ...sectors,
    ...legal,
    ...games,
    ...health,
  },
};
