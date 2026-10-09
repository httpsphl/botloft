// Roles for particular kinds of business (spec 26.3), listed under Business:
// what the owner reads about each. None sends, signs, buys, pays, refunds or
// publishes by itself.

export const sectors = {
  "real-estate-assistant": {
    name: "Real Estate Assistant",
    role: "Helps you buy, sell or rent a property: compares listings, prepares questions and checklists, and drafts ads and messages",
    summary:
      "Helps you buy, sell or rent a property: compares listings, prepares questions and drafts the ad",
    about:
      "Compares places in one table with the same facts for each, including the real costs, and tells you what a listing leaves out. It prepares visit checklists and questions, and helps you write an honest ad and questions for buyers or tenants. It never states a market value as a fact, makes an offer or signs, and it is not an agent or a lawyer.",
    when: "When you are looking for a place, or getting yours ready to sell or rent.",
    pairs: "Researcher, for facts about an area, and Writer, to polish the ad.",
  },
  "hospitality-guest-assistant": {
    name: "Guest Experience Assistant",
    role: "Helps a hotel, guesthouse or rental host look after guests: answers, welcome messages, guides, and notes on what to improve",
    summary:
      "Helps a hotel or rental host look after guests: answers, welcome messages, guides and improvements",
    about:
      "Keeps a short sheet of facts about your place and drafts replies to guests from it, never inventing a time, a price or a promise. It writes the welcome message and guest guide, drafts calm answers to complaints and reads your reviews for what guests like and what repeats as a problem. Refunds and discounts stay your decision.",
    when: "When guests ask the same things all day, or you want better reviews.",
    pairs: "Translator, for guests who speak another language, and Social Media.",
  },
  "returns-assistant": {
    name: "Returns and Refunds Assistant",
    role: "Handles customer returns and refund requests by your own policy: sorts them, drafts the replies and spots patterns and abuse",
    summary:
      "Handles returns and refund requests by your own policy: sorts them, drafts replies and spots patterns",
    about:
      "Reads each request against your written policy and sorts it as inside, outside or a doubt, then drafts a clear, friendly reply. Doubts and exceptions come to you. Each week it shows what repeats, such as items with many returns, without accusing any customer. It warns you when a customer may have a legal right to a refund. It never issues a refund itself.",
    when: "When returns and refund requests take too much of your day.",
    pairs: "Customer Support, for tone, and E-commerce Manager, for listing fixes.",
  },
  "supply-chain-planner": {
    name: "Supply Chain Planner",
    role: "Plans what to buy and when: stock levels, reorder points, supplier comparisons and delivery risks, for a small business",
    summary:
      "Plans what to buy and when: stock levels, reorder points and supplier comparisons for a small business",
    about:
      "Works from your sales, stock, prices and lead times to propose a reorder point and an order size for each item, with the formula shown. It flags the items you cannot run out of and the slow ones that tie up money, compares suppliers on the full cost and lists the risks with a plan B. It never places an order or contacts a supplier.",
    when: "When you run out of stock, or hold too much of what does not sell.",
    pairs: "Bookkeeper, for costs, and Cash Flow Forecaster, for what you can afford.",
  },
  "grant-writer": {
    name: "Grant Writer",
    role: "Finds and prepares applications for grants and public calls: reads the rules, plans the answer and drafts it from your real facts",
    summary:
      "Prepares applications for grants and public calls: reads the rules, plans the answer and drafts it",
    about:
      "Reads the call in full, summarizes it with the deadline first and checks honestly whether you qualify before it writes anything. Then it plans the application against the scoring and drafts each section from your real facts, marking every gap as a question. It never invents a result, a partner or a number, and never submits for you.",
    when: "When you find a grant, a public call or a funding program that could fit you.",
    pairs: "Financial Analyst, for the budget, and Editor, to polish.",
  },
  "restaurant-manager": {
    name: "Restaurant Assistant",
    role: "Helps run a small restaurant or café: menu and costs, shifts, orders and supplies, and answers to guests",
    summary:
      "Helps run a small restaurant or café: menu costs, shifts, supplies and answers to guests",
    about:
      "Works out what each dish really costs and where you lose money, plans shifts around your busy hours, keeps the shopping list and a daily sheet that shows what repeats. It never says a dish is free of an allergen or safe for a diet on a guess: that comes from your own recipe sheets, and when they do not say, it tells you to check first.",
    when: "When you run a small food business and the paperwork eats your time.",
    pairs: "Bookkeeper, for costs, and Social Media, for posts about the menu.",
  },
  "course-creator": {
    name: "Online Course Creator",
    role: "Helps you turn what you know into an online course or workshop: the promise, the outline, the lessons, the exercises and the page that explains it",
    summary:
      "Helps you turn what you know into an online course: outline, lessons, exercises and the sales page",
    about:
      "Starts from the student and an honest promise you can deliver, builds the outline backwards from the result and drafts the lessons and exercises in your voice, with your own examples. The course page uses only real content and results: no made-up testimonials, no false deadlines, no earnings promises. You publish and set the price.",
    when: "When you know something people would pay to learn and want to teach it.",
    pairs:
      "Presentation Designer, for slides, Video Scriptwriter, for scripts, and Editor, to proofread.",
  },
};
