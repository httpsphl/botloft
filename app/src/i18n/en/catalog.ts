import { base } from "./catalogBase";
import { contentAndLearning } from "./catalogContent";
import { design } from "./catalogDesign";
import { engineering } from "./catalogEngineering";
import { engineeringMore } from "./catalogEngineeringMore";
import { finance } from "./catalogFinance";
import { marketing } from "./catalogMarketing";
import { paidMedia } from "./catalogPaidMedia";
import { people } from "./catalogPeople";
import { product } from "./catalogProduct";

// The Bot agency (spec 26): ready-made bots the owner adds to a crew. The
// roles are the daemon's sheets, by id; this is what the owner reads about
// each. Plain words: "bot" and "crew", never "template" or "prompt".

export const catalog = {
  title: "Bot agency",
  open: "Bot agency",
  openHint: "Ready-made bots you can add to this crew",
  intro:
    "Pick a bot for the job. It joins this crew ready to work, and you can change anything about it afterwards.",
  invite: {
    title: "Who do you want on your crew?",
    body: "Not sure what to create? Pick a bot for the job. You can change anything about it afterwards.",
  },
  search: "Search bots",
  filter: "Kinds of bots",
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
  },
  none: "No bot matches that.",
  loading: "Loading the bots…",
  add: "Add to crew",
  learnMore: "Learn more",
  back: "Back to all bots",
  seeAll: "See all bots",
  joined: (name: string) => `${name} joined the crew`,
  customize: "Customize",
  failed: {
    load: "Could not load the bots",
    add: "Could not add the bot",
  },
  detail: {
    what: "What it does",
    when: "When to call it",
    pairs: "Works well with",
    technical: "What the bot is told when it starts",
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
  },
};
