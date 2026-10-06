// Product and management roles (spec 26.3): what the owner reads about each.

export const product = {
  "product-manager": {
    name: "Product Manager",
    role: "Turns ideas and feedback into a clear plan: what to build, in what order and why",
    summary: "Decides what to build next and writes it down so others can build it",
    about:
      "Looks at your ideas, feedback and numbers and helps you choose what is worth building next. It writes each choice as a short brief that the rest of the crew can build from.",
    when: "When you have more ideas than time and need to pick.",
    pairs: "Business Analyst, to detail each item, and Developer and Designer, who build it.",
  },
  "project-manager": {
    name: "Project Manager",
    role: "Plans a project, tracks who does what by when and flags what is late or blocked",
    summary:
      "Breaks a project into steps, tracks progress and warns you early when something slips",
    about:
      "Turns a goal into a plan with steps, owners and dates, keeps it up to date and tells you early when something is late or stuck.",
    when: "For anything with several steps and people: a launch, a renovation, a move, a campaign.",
    pairs: "Agile Facilitator, for the rhythm of the work, and Meeting Secretary.",
  },
  "business-analyst": {
    name: "Business Analyst",
    role: "Turns a vague request into clear requirements, questions and criteria for when it is done",
    summary: "Asks the right questions and writes down exactly what needs to be built",
    about:
      "Takes a vague request like 'I need a way to track orders' and turns it into precise requirements, the exceptions and how everyone will know it is right.",
    when: "Before building anything that is not yet clear.",
    pairs:
      "Product Manager, who sets the priorities, and Developer and QA Tester, who use the requirements.",
  },
  "ux-researcher": {
    name: "UX Researcher",
    role: "Plans how to learn what users need and turns interviews and feedback into findings",
    summary: "Finds out what your users really need, from their words and their behavior",
    about:
      "Plans interviews, surveys and tests, and turns the notes and answers you bring into findings with evidence. It cannot talk to anyone itself: it prepares and analyzes.",
    when: "Before a big decision about a product, a page or a service.",
    pairs:
      "Product Manager and Designer, who use the findings, and Data Analyst, for survey numbers.",
  },
  "agile-facilitator": {
    name: "Agile Facilitator",
    role: "Runs planning, quick check-ins and retrospectives, and keeps the crew's work flowing",
    summary:
      "Keeps the crew's work moving in small steps, with regular check-ins and honest looks back",
    about:
      "Keeps a simple board of what is next, in progress and done, asks the bots for quick updates and leads a look back at the end of each round to change one thing.",
    when: "When the crew has many bots and work keeps getting stuck or forgotten.",
    pairs: "Project Manager, for the plan, and Meeting Secretary.",
  },
  "goals-coach": {
    name: "Goals Coach",
    role: "Helps set clear goals and measures, and checks progress against them",
    summary: "Turns what you want into goals you can measure, and checks how you are doing",
    about:
      "Helps you say what you want in a way that can be checked, picks a few measures that show progress and looks at the numbers with you regularly, being honest when you are off track.",
    when: "When you want to grow something and are not sure it is working.",
    pairs: "Data Analyst, who gets the numbers, and Project Manager.",
  },
  "meeting-secretary": {
    name: "Meeting Secretary",
    role: "Prepares agendas, takes notes and turns meetings into decisions and action items",
    summary: "Prepares the meeting, writes it up and turns it into decisions and things to do",
    about:
      "Prepares agendas and, from the notes or the transcript you give it, writes up the decisions, the action items and the open questions. It does not join calls and it sends nothing by itself.",
    when: "When meetings end with no clear record of who will do what.",
    pairs: "Project Manager and Agile Facilitator, who follow up the action items.",
  },
  "process-analyst": {
    name: "Process Analyst",
    role: "Maps how work is done today, finds waste and writes the better way as clear steps",
    summary: "Draws how work really happens, finds what wastes time and writes a simpler way",
    about:
      "Maps a process as it really works, finds repeated steps, waits and errors, and writes a simpler version as a checklist that a new person could follow.",
    when: "When the same work is slow, error-prone or lives only in someone's head.",
    pairs: "Operations Manager and Data Analyst, who help measure it.",
  },
  "operations-manager": {
    name: "Operations Manager",
    role: "Keeps the day-to-day running: routines, checklists, suppliers and what keeps slipping",
    summary:
      "Keeps the everyday running smoothly: routines, checklists and the things that keep slipping",
    about:
      "Keeps track of what must happen daily, weekly and monthly, turns what is in your head into checklists, watches renewals and supplies and warns you before something is missed. It can propose routines that run on a schedule.",
    when: "When running the business is eating the time you need to grow it.",
    pairs: "Process Analyst, who improves the routines, and Personal Assistant.",
  },
  recruiter: {
    name: "Recruiter",
    role: "Writes job posts, sorts applications against clear criteria and prepares interviews",
    summary:
      "Writes the job post, sorts applications by clear criteria and prepares good interview questions",
    about:
      "Writes the job post, defines what a good candidate looks like, compares the applications you give it against those criteria and prepares the same interview questions for everyone. It recommends, you decide, and it never contacts candidates.",
    when: "When you are hiring and want a fair, organized process.",
    pairs: "Writer, to polish the post, and Researcher, to look up the market.",
  },
  "onboarding-coach": {
    name: "Onboarding Coach",
    role: "Prepares a new person's first weeks: what to learn, who to meet and what to do first",
    summary: "Plans a new person's first weeks so they get useful quickly and feel welcome",
    about:
      "Plans the first day, week and month of someone new, writes the welcome note and a guide to how things work, and lists what must be ready before they start.",
    when: "When someone is joining and you want them useful quickly.",
    pairs: "Process Analyst, who maps the work, and Writer, to polish the guide.",
  },
  "customer-success": {
    name: "Customer Success",
    role: "Keeps customers getting value: checks in, spots those at risk and plans the next step",
    summary: "Helps customers get real value, notices who is unhappy and suggests what to do next",
    about:
      "Keeps a simple table of your customers, notices the ones at risk (less use, late payments, complaints) and drafts a short, warm message for each one that needs attention. It sends nothing without your approval.",
    when: "When you have recurring customers and want fewer of them to leave.",
    pairs: "Customer Support, for questions, and Data Analyst, for usage numbers.",
  },
  "event-planner": {
    name: "Event Planner",
    role: "Plans an event from start to finish: budget, schedule, suppliers and a checklist",
    summary: "Plans your event from budget to the last checklist item, so nothing is forgotten",
    about:
      "Plans events of any size: a budget by line, a timeline counted back from the date, supplier options to compare, the schedule for the day and a checklist for the day after. It does not book or invite anyone.",
    when: "For a launch, a party, a workshop or a family gathering.",
    pairs: "Researcher, to compare suppliers, and Writer, for invitations.",
  },
  "travel-planner": {
    name: "Travel Planner",
    role: "Plans a trip: route, places to stay, daily schedule and budget, with options and what to book",
    summary:
      "Plans your trip with options, a day-by-day schedule and a budget, and tells you what to book first",
    about:
      "Researches routes, stays and activities, proposes options, builds a day-by-day plan with realistic times and a budget, and lists what to book and by when. It never books and never enters your documents or cards anywhere.",
    when: "For a holiday, a work trip or a family visit.",
    pairs: "Researcher, for a deeper look at a place, and Data Analyst, to compare costs.",
  },
};
