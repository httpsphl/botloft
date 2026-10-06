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
  },
  none: "No bot matches that.",
  loading: "Loading the bots…",
  add: "Add to crew",
  learnMore: "Learn more",
  back: "Back to all bots",
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
    developer: {
      name: "Developer",
      role: "Writes and changes code, runs the tests and explains what changed",
      summary: "Builds and fixes software, checks its own work and says what it did",
      about:
        "Writes new features, fixes bugs and tidies up code in your project. It runs the tests after each change and tells you in plain words what it did and what is still open.",
      when: "When you have a website, an app or a script to build or fix.",
      pairs:
        "Code Reviewer and QA Tester, who check its work, and Designer, who gives it screens to build.",
    },
    "code-reviewer": {
      name: "Code Reviewer",
      role: "Reviews what other bots changed and points out bugs, risks and needless complexity",
      summary: "Reads changes with a critical eye and says what is wrong or risky",
      about:
        "Reads the changes another bot made and lists what could break, what is risky and what is more complicated than it needs to be. It never edits the code itself, it only reports, so you decide. It thinks harder than most bots, so it uses a bit more of your plan.",
      when: "After a Developer changes something that matters: a payment, a login, anything hard to undo.",
      pairs: "Developer, whose work it reviews, and QA Tester.",
    },
    "qa-tester": {
      name: "QA Tester",
      role: "Tests what was built, tries to break it and reports how to repeat each problem",
      summary: "Uses the work like a person would and writes down how to repeat each problem",
      about:
        "Tries what was built the way a real person would, then the way a careless one would, and reports each problem with the steps to repeat it. It does not fix things, it finds them.",
      when: "Before you show or ship something, or when people say that it broke.",
      pairs: "Developer, who fixes what it finds, and Code Reviewer.",
    },
    designer: {
      name: "Designer",
      role: "Designs screens and pages in HTML that you see live and can ask to change",
      summary: "Designs pages and screens you can watch take shape, then refines them with you",
      about:
        "Designs pages and screens as HTML. You watch each one take shape in the design area while it works, and ask for changes the way you would with a person.",
      when: "For a landing page, an app screen, a menu, a card or any visual idea you want to see before building it.",
      pairs: "Developer, who can build what it designs, and Writer, for the words on the page.",
    },
    writer: {
      name: "Writer",
      role: "Writes articles, e-mails and pages in your tone of voice",
      summary: "Writes clear texts that sound like you and fit who will read them",
      about:
        "Writes articles, e-mails, product texts and pages in your tone of voice. Give it a sample of your own writing and it matches it.",
      when: "When the words need to be clear and sound like you.",
      pairs: "Researcher, for facts, and Translator, for other languages.",
    },
    "social-media": {
      name: "Social Media",
      role: "Plans content and writes posts for each network, but never publishes on its own",
      summary: "Comes up with ideas, a calendar and posts for you to approve",
      about:
        "Plans what to post and writes the posts for each network. It never publishes on its own: you approve every post.",
      when: "When you want a steady presence without starting from a blank page each time.",
      pairs: "Designer, for the images, and Writer, for longer texts.",
    },
    translator: {
      name: "Translator",
      role: "Translates and adapts texts between languages, keeping the tone and flagging what gets lost",
      summary: "Translates so it reads naturally and tells you where the meaning shifts",
      about:
        "Translates texts and adapts them for the people who will read them, keeping the tone and telling you where meaning is lost.",
      when: "For a website in another language, an e-mail to a client abroad or a document you cannot read.",
      pairs: "Writer, to rework a text that needs more than a translation.",
    },
    researcher: {
      name: "Researcher",
      role: "Researches a question on the web, checks the sources and reports what it found and what it did not",
      summary:
        "Looks things up on the web, checks the sources and gives you a short answer with links",
      about:
        "Looks things up on the web with its own browser, which you can watch, compares sources and gives you a short answer with the links. It says what it could not find.",
      when: "For prices, competitors, rules, how-tos or any question you would otherwise spend an hour searching.",
      pairs: "Writer and Data Analyst, who use what it finds.",
    },
    "data-analyst": {
      name: "Data Analyst",
      role: "Reads spreadsheets and data, does the math, shows the work and explains what it means",
      summary:
        "Turns spreadsheets and numbers into answers, shows how it got there and draws charts",
      about:
        "Reads spreadsheets and data, does the math, shows how it got each result and explains what it means, with charts you can see.",
      when: "When you have numbers (sales, expenses, results) and want answers, not tables.",
      pairs: "Researcher, for facts from outside your data.",
    },
    "sales-prospector": {
      name: "Sales Prospector",
      role: "Finds and qualifies possible customers and drafts the messages, but sends nothing without you",
      summary:
        "Finds people and companies that fit, sorts them and drafts first messages for you to approve",
      about:
        "Finds people and companies that fit what you sell, explains why, and drafts the first message to each. It sends nothing without your approval. It works better with a connected tool for the network where you look for customers.",
      when: "When you need a list of good leads and a start for each conversation.",
      pairs: "Writer, to polish the messages, and Researcher, to dig into a company.",
    },
    "customer-support": {
      name: "Customer Support",
      role: "Answers customers' questions from the material you gave it and passes on what it cannot answer",
      summary:
        "Answers questions kindly using your own material and hands you what it cannot answer",
      about:
        "Answers customers' questions from the material you give it and hands you the cases it cannot solve. It drafts the replies and waits for your approval at first.",
      when: "When the same questions keep coming and you want help answering them well.",
      pairs: "Developer, for technical problems, and Writer, for wording.",
    },
    "personal-assistant": {
      name: "Personal Assistant",
      role: "Keeps your tasks in order, summarizes what you need to read and drafts e-mails",
      summary: "Keeps your tasks organized, sums up what you need to read and drafts your e-mails",
      about:
        "Keeps your tasks in order, summarizes what you need to read, drafts e-mails and reminds you of what matters. Calendar and e-mail only work if you connect a tool for them.",
      when: "When your day is full of small things that pile up.",
      pairs: "Researcher and Writer, for bigger jobs.",
    },
  },
};
