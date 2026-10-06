// Marketing and sales roles (spec 26.3): what the owner reads about each.

export const marketing = {
  "seo-specialist": {
    name: "SEO Specialist",
    role: "Helps a site show up in search: finds what people look for, fixes pages and plans content",
    summary: "Finds what people search for and improves your pages so they get found",
    about:
      "Finds the words people really use to look for what you sell, checks your pages for what holds them back and suggests changes to titles, text and structure. It only uses honest methods, and results take weeks.",
    when: "When your site is up but people are not finding it.",
    pairs: "Writer, to write the pages, and Developer, to fix technical problems.",
  },
  "email-marketer": {
    name: "Email Marketer",
    role: "Plans and writes email campaigns and sequences, with a clear goal and respect for consent",
    summary:
      "Writes emails and sequences that get opened and answered, with consent and an easy way out",
    about:
      "Plans and writes emails and automatic sequences (welcome, follow-up, win-back, announcements) with subject-line options and a clear action. It never sends anything and it reminds you that email laws apply.",
    when: "When you have a list of people who agreed to hear from you and want to use it well.",
    pairs: "Designer, for the layout, and Data Analyst, to read the results.",
  },
  "content-strategist": {
    name: "Content Strategist",
    role: "Plans what to publish, for whom and why, so that content supports a business goal",
    summary: "Plans what to publish, for whom and why, with a calendar tied to your goals",
    about:
      "Chooses a few themes you can speak about with real knowledge, and builds a calendar you can keep: each piece with its goal, its format and the question it answers. Then it writes a short brief for each.",
    when: "When you post a lot but do not know if it leads anywhere.",
    pairs: "Writer and Video Scriptwriter, who make the pieces, and Researcher, for facts.",
  },
  copywriter: {
    name: "Copywriter",
    role: "Writes persuasive, honest copy for ads, landing pages and product pages",
    summary: "Writes headlines and copy that make people act, without exaggerating",
    about:
      "Writes headlines, ads, landing pages and product descriptions with several options, each built on what the buyer cares about. It keeps claims you can prove and never makes up testimonials or numbers.",
    when: "When a page or an ad is not getting people to act.",
    pairs: "SEO Specialist, for search words, and Designer, for the layout.",
  },
  "ads-manager": {
    name: "Ads Manager",
    role: "Plans paid ad campaigns, writes the ads and reads the results, but never spends money by itself",
    summary:
      "Plans your ads, writes them and reads the results, and never spends a cent without you",
    about:
      "Plans a small start for your paid ads, writes them for each platform and, when you give it the numbers, tells you what to stop, keep and test. It never creates or pays for an ad: you do, with its plan.",
    when: "When you want to try paid ads without wasting money.",
    pairs: "Copywriter, for the text, and Data Analyst, for the numbers.",
  },
  "video-scriptwriter": {
    name: "Video Scriptwriter",
    role: "Writes scripts and shot lists for short and long videos",
    summary:
      "Writes video scripts with a strong start, a clear structure and a list of shots to film",
    about:
      "Writes scripts in two columns, what is said and what is seen, with a strong first few seconds and a list of what you need to film. It does not make the video.",
    when: "For a short video, a product demo or a tutorial.",
    pairs:
      "Content Strategist, for the plan around the video, and Copywriter, for the call to action.",
  },
  "newsletter-editor": {
    name: "Newsletter Editor",
    role: "Plans and writes a recurring newsletter: topics, issues and a consistent voice",
    summary: "Plans and writes your newsletter issue by issue, in a voice readers recognize",
    about:
      "Defines what your newsletter promises, keeps a plan of the next issues and writes each one in a steady voice. It checks facts, credits its sources and never sends an issue by itself.",
    when: "When you want a newsletter but not the weekly blank page.",
    pairs: "Researcher, for facts, and Designer, for images.",
  },
  "community-manager": {
    name: "Community Manager",
    role: "Drafts replies, welcomes members and flags problems in your community, following rules you set",
    summary: "Prepares replies and moderation for your community, following the rules you set",
    about:
      "Drafts replies and welcome messages in your community's voice and flags what needs you: complaints, harassment, safety or legal matters. It never posts, deletes or bans anyone on its own.",
    when: "When your group or page is growing faster than you can answer.",
    pairs:
      "Customer Support, for customer problems, and Content Strategist, for conversation ideas.",
  },
  "ecommerce-manager": {
    name: "E-commerce Manager",
    role: "Looks after an online store: product pages, catalog, promotions and what to fix first",
    summary: "Improves your online store: product pages, catalog, promotions and what to fix first",
    about:
      "Goes through your product pages like a buyer, rewrites titles and descriptions, looks at what sells and what does not and plans promotions your margin can afford. It never touches the store itself.",
    when: "When you have an online store and want more sales from it.",
    pairs: "Copywriter, for text, and SEO Specialist, for search.",
  },
  "brand-strategist": {
    name: "Brand Strategist",
    role: "Defines what a brand stands for, how it sounds and how it differs from others",
    summary: "Defines who your brand is, how it speaks and what makes it different",
    about:
      "Asks about your story, looks at how similar businesses present themselves and writes a short guide: who you serve, your promise, your values, your voice with examples, and what makes you different.",
    when: "When you are starting out, or when what you say sounds like everyone else.",
    pairs: "Designer, Copywriter and Content Strategist, who follow the guide.",
  },
  "competitor-analyst": {
    name: "Competitor Analyst",
    role: "Studies competitors from public information and shows where you stand and where you can win",
    summary: "Studies your competitors from public sources and shows where you can stand out",
    about:
      "Reads your competitors' sites, prices, offers and reviews, compares them with you on what customers care about and suggests moves. Everything has its source and date, and it only uses public information.",
    when: "Before you set prices, launch something or change how you present yourself.",
    pairs: "Pricing Analyst and Brand Strategist, who use what it finds.",
  },
  "growth-marketer": {
    name: "Growth Marketer",
    role: "Designs small experiments to get more customers and checks which ones worked",
    summary: "Runs small, measured experiments to find what brings more customers",
    about:
      "Finds the biggest leak between a stranger and a customer and proposes one small experiment at a time, with what to measure and how long to wait. It tells you honestly when a result is just noise.",
    when: "When you want more customers and do not know what to try first.",
    pairs: "Data Analyst, for the numbers, and Ads Manager or Email Marketer, to run a test.",
  },
  "conversion-optimizer": {
    name: "Conversion Optimizer",
    role: "Finds why visitors do not buy or sign up and proposes fixes to test",
    summary: "Finds why visitors leave without buying and suggests changes to test",
    about:
      "Walks through your page as a stranger on a phone, finds what makes people doubt or give up and proposes specific changes, ordered by expected effect, each one to be tested. No tricks like fake countdowns.",
    when: "When you get visits but few sales or sign-ups.",
    pairs: "Copywriter and Designer, who make the changes, and Data Analyst, to read the tests.",
  },
  "pr-writer": {
    name: "PR Writer",
    role: "Writes press releases, pitches and statements that are accurate and newsworthy",
    summary: "Writes press releases and pitches that journalists can use, and that are true",
    about:
      "Writes press releases, pitches to journalists and statements, and tells you when there is no real news. Every fact comes from you or a named source, and it sends nothing to anyone.",
    when: "For a launch, an award, an event or a delicate moment you must explain.",
    pairs: "Researcher, to find outlets, and Writer, to polish.",
  },
  "reputation-manager": {
    name: "Reputation Manager",
    role: "Reads reviews and mentions, drafts honest replies and spots patterns to fix",
    summary: "Reads your reviews, drafts honest replies and shows what customers keep saying",
    about:
      "Sorts your reviews, drafts a reply to each one that needs it and shows what customers keep praising or complaining about, so you can fix the cause. It never argues, never writes fake reviews and never posts by itself.",
    when: "When reviews pile up and you do not have time to answer them well.",
    pairs: "Customer Support and Operations Manager, to fix real problems.",
  },
  "proposal-writer": {
    name: "Proposal Writer",
    role: "Writes clear proposals and quotes that match what the client asked for",
    summary:
      "Writes proposals and quotes that answer what the client asked and are easy to say yes to",
    about:
      "Writes the client's problem back to them, lists exactly what is and is not included, lays out the steps, the price and the terms, and makes it easy to accept. It asks you for the numbers instead of inventing them.",
    when: "When a client asks for a quote and you want to answer well and fast.",
    pairs: "Pricing Analyst, for the price, and Sales Coach, for the conversation.",
  },
  "sales-coach": {
    name: "Sales Coach",
    role: "Prepares for sales conversations: questions, objections and follow-ups, with practice",
    summary: "Helps you prepare sales calls, handle objections and follow up well",
    about:
      "Prepares you for a sales conversation with the right questions and honest answers to the objections you expect, plays the customer so you can practice and writes the follow-up. It never helps pressure anyone.",
    when: "Before an important call, or when you keep losing the same kind of sale.",
    pairs: "Researcher, to learn about the customer, and Proposal Writer.",
  },
  "pricing-analyst": {
    name: "Pricing Analyst",
    role: "Helps set prices from costs, competitors and what customers value, and tests options",
    summary: "Helps you set prices from your costs, the market and what customers value",
    about:
      "Works out your real cost and margin, compares what others charge and what they include, proposes a price range with the reasons and a small test to try. You always set the price.",
    when: "When you are launching something, or suspect you charge too little.",
    pairs: "Competitor Analyst, for the market, and Data Analyst, for sales numbers.",
  },
};
