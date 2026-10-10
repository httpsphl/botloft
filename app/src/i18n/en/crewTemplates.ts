// Crew templates (spec 29.1): ready-made teams the owner starts a crew from.
// The roles of each are in `features/crews/crewTemplates.ts`; this is what the
// owner reads. `goal` becomes the chief's instructions, so it says that the
// team is already there.

export const crewTemplates = {
  title: "Start from a template",
  intro: "Pick a team that is ready to work, or start with an empty crew.",
  scratch: "Start with an empty crew",
  scratchHint: "Only the chief. You add the agents yourself.",
  bots: (count: number) => (count === 1 ? "1 agent" : `${count} agents`),
  /** Above the list of bots a team adds. */
  adds: "This team adds",
  change: "Pick another template",
  back: "Back",
  /** After the crew is made but some bots could not be added. */
  partial: (failed: number) =>
    failed === 1
      ? "The crew was created, but 1 agent could not be added. You can add it from the Agent agency."
      : `The crew was created, but ${failed} agents could not be added. You can add them from the Agent agency.`,
  loadFailed: "Could not load the templates",
  items: {
    "content-studio": {
      name: "Content studio",
      summary: "Plans, writes, edits and posts content for a site, a brand or a channel.",
      goal: "Run a small content studio: plan what to publish, get it written and edited, and keep the social channels moving. The team is already here, so give them work as tasks and tell me what you need.",
    },
    "software-team": {
      name: "Software team",
      summary: "Designs, builds, tests, reviews and documents a piece of software.",
      goal: "Build and keep up my software: plan the work, split it between the developers, have every change tested and reviewed, and keep the docs true. The team is already here, so give them work as tasks.",
    },
    "online-store": {
      name: "Online store",
      summary: "Runs the listings, the ads, the customers, the returns and the books of a shop.",
      goal: "Help me run my online store: keep the listings and the ads in good shape, answer customers and returns by my policy, and keep the books. The team is already here, so give them work as tasks.",
    },
    "growth-team": {
      name: "Growth team",
      summary: "Finds new customers with ads and email, and checks that the numbers are right.",
      goal: "Grow my customer base: plan small, careful tests of ads and email, make sure results are measured correctly and tell me what to stop, keep and try next. The team is already here, so give them work as tasks.",
    },
    "research-desk": {
      name: "Research desk",
      summary: "Finds, checks and explains what is known about a topic, with real sources.",
      goal: "Answer my questions with research I can trust: find sources, check the facts, run the numbers and write clear reports. The team is already here, so give them work as tasks.",
    },
    "back-office": {
      name: "Small business back office",
      summary: "Keeps the books, the bills, the invoices, the cash and the contracts in order.",
      goal: "Keep my business paperwork in order: the books, the bills, the invoices, the cash forecast, the taxes and the contract dates. Nothing is paid or sent without me. The team is already here, so give them work as tasks.",
    },
    "personal-office": {
      name: "Personal office",
      summary: "Looks after your tasks, trips, bills, taxes, goals and meetings.",
      goal: "Be my personal office: keep my tasks, trips, bills and taxes in order, keep track of my goals and write up my meetings. Nothing is sent or paid without me. The team is already here, so give them work as tasks.",
    },
    "course-studio": {
      name: "Course studio",
      summary: "Turns what you know into an online course, with videos, slides and promotion.",
      goal: "Turn what I know into an online course: plan it, write the lessons, make the scripts and slides and prepare the promotion. The team is already here, so give them work as tasks.",
    },
    "game-studio": {
      name: "Game studio",
      summary: "Designs a game: the rules, the story, the levels, the numbers and the testing.",
      goal: "Design my game: shape the rules, write the story, plan the levels, balance the numbers and learn from the playtests. The team is already here, so give them work as tasks.",
    },
    "people-team": {
      name: "People team",
      summary: "Hires, welcomes, trains and listens to a team, without deciding for you.",
      goal: "Help me look after my team: hiring, onboarding, training, policies, feedback and surveys. The decisions about people are mine. The team is already here, so give them work as tasks.",
    },
  },
};
