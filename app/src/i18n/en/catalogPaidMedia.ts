// Paid media roles, listed under Marketing & sales (spec 26.3): what the owner
// reads about each. None creates, changes or pays for an ad.

export const paidMedia = {
  "ppc-strategist": {
    name: "Search Ads Strategist",
    role: "Plans search ad campaigns: the keywords, the structure, the ads and a small budget to test, with a plan you carry out",
    summary:
      "Plans your search ads, from keywords to the ads themselves, with a small budget to test first",
    about:
      "Starts from what a customer is worth to you, builds the keyword groups by what people intend, writes the ads from your real offer and sets a small test with clear rules for when to stop and when to raise. It never creates or pays for an ad: you do, with its plan.",
    when: "When you want to start search ads, or the ones you have are not paying off.",
    pairs: "Copywriter, to sharpen the ads, and Search Query Analyst, to read the search terms.",
  },
  "paid-social-strategist": {
    name: "Paid Social Strategist",
    role: "Plans paid campaigns on social platforms: the audience, the formats, the creative needs and a test budget, without sensitive targeting",
    summary:
      "Plans your paid social campaigns: who to reach, what to show and how to test it with a small budget",
    about:
      "Picks one objective, defines who to reach by interests, place and the customer lists you choose, and plans the formats and the variants to test. It never targets by sensitive traits like health or religion and never preys on insecurities. You create and pay for the campaign yourself.",
    when: "When you want to reach new people on social platforms without wasting your budget.",
    pairs:
      "Ad Creative Strategist, for what to say and show, and Video Scriptwriter, for short videos.",
  },
  "ad-creative-strategist": {
    name: "Ad Creative Strategist",
    role: "Plans what ads should say and show: angles, hooks and variants to test, and learns from what performs",
    summary:
      "Plans what your ads should say and show, with a few angles to test and what each one teaches you",
    about:
      "Takes the words your customers use and writes three or four different angles, each with the hook, the text and a brief for the picture or video. It changes one thing per test, so a result says why it won. It only uses claims you can prove: no invented reviews or false scarcity.",
    when: "When your ads all look the same, or you do not know what to test next.",
    pairs: "Designer, for the pictures, Copywriter, to polish, and Paid Social Strategist.",
  },
  "ad-auditor": {
    name: "Ad Account Auditor",
    role: "Reviews how your ad campaigns are set up and run, finds wasted spend and missed chances, and lists fixes in order",
    summary:
      "Reviews your ad campaigns, finds wasted spend and missed chances, and lists the fixes in order",
    about:
      "Reads the reports you give it and finds the usual leaks: spend with no results, people reached twice, tired ads, searches that do not fit and results counted wrong. Each finding comes with the numbers, the money at stake and the fix, in order. It has no access to your account and changes nothing.",
    when: "When your ad spend is growing but you are not sure it is working.",
    pairs:
      "Tracking Specialist, to check the counts, and Search Ads Strategist, to apply the fixes.",
  },
  "tracking-specialist": {
    name: "Tracking Specialist",
    role: "Checks that your ads and site measure results correctly: events, tags, links and what counts as a result",
    summary:
      "Checks that your ads and site count results correctly, so you can trust the numbers you decide on",
    about:
      "Lists every place a result is counted and compares them: ad platform, site, store and spreadsheet. It explains each gap in plain words, such as double counting or links that lose their tags, and proposes one clean naming for events and links. It says how far the numbers can be trusted and collects no more about visitors than you need.",
    when: "Before you spend on ads, or when the numbers from different places do not agree.",
    pairs: "Frontend Developer, who adds the tags, and Ad Account Auditor.",
  },
  "search-query-analyst": {
    name: "Search Query Analyst",
    role: "Reads the real searches behind your ads or site, groups them by intent and finds what to add, exclude or write for",
    summary:
      "Reads the real searches behind your ads and site, and tells you what to add, exclude or write for",
    about:
      "Takes the real words people typed and groups them by what they want: to buy, to compare, to learn. It gives you three lists: what to exclude because it wastes money, what to add because it works and what to write about because people ask. It says how sure it is, since a term with three clicks proves nothing.",
    when: "When you run search ads or want to know what people look for before they find you.",
    pairs: "Search Ads Strategist, who uses the lists, and SEO Specialist, for the content ideas.",
  },
};
