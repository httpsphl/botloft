// What the agents say and show in the site's demos (spec 39.4). The names
// are the sample crew's, the same in every language; the rest is made up
// for the scenes and reads as the app does.

export const siteDemos = {
  window: {
    crew: "Research",
    agents: "4 agents",
    message: "Message Scout",
  },
  states: {
    idle: "Idle",
    working: "Working",
    approval: "Needs approval",
    sleeping: "Asleep",
  },
  roles: {
    scout: "Finds sources and summarizes them",
    writer: "Turns notes into posts",
    analyst: "Crunches the numbers",
    designer: "Draws the shop's pages",
  },
  hero: {
    ask: "Find three good sources on sourdough starters and hand them to Writer for Friday's post.",
    wants: "Scout wants to use the browser on",
    site: "bakers.example",
    allow: "Allow",
    once: "Only this time",
    deny: "Deny",
    allowed: "You allowed bakers.example",
    steps: "Took 4 steps in the browser",
    reply:
      "Found three: a baker's guide to feeding a starter, a food-science piece on wild yeast and a home baker's notes. Handing them to Writer.",
    sent: "Sent Writer a task",
    task: "Draft Friday's post from these three sources",
  },
  browser: {
    live: "Live",
    tab: "Recipes",
    page: "Find a recipe",
    query: "sourdough starter",
    search: "Search",
    results: [
      "Feeding a starter, day by day",
      "Why your starter smells sour",
      "Rye or wheat to begin?",
    ],
    control: "Scout is in control",
    take: "Take control",
  },
  ask: {
    wants: "Writer wants to run a command",
    command: "npm run build",
    allow: "Allow",
    deny: "Deny",
    allowed: "You allowed it",
    after: "The build passed. Publishing the draft for your review.",
  },
  handoff: {
    task: "Draft Friday's post",
  },
  routines: {
    items: [
      { name: "Morning summary", when: "Weekdays, 8:00" },
      { name: "Link check", when: "Every 2 hours" },
      { name: "Weekly numbers", when: "Mondays, 9:00" },
    ],
    running: "Running",
    done: "Done, 8:00",
  },
  phone: {
    time: "9:41",
    notice: "Writer needs you",
    noticeBody: "Wants to run npm run build",
    allow: "Allow",
    deny: "Deny",
    allowed: "Allowed",
  },
  maker: {
    ask: "Set up a crew for my bakery's online shop.",
    plan: "The plan",
    items: [
      "Crew: Bakery shop, with a Chief",
      "Designer: draws the shop's pages",
      "Clerk: takes the orders",
      "Routine: orders summary, weekdays 9:00",
    ],
    build: "Build it",
    change: "Change",
    built: "Built. Undo takes it back.",
  },
  sleep: {
    message: "New message",
  },
};
