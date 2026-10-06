// Engineering roles (spec 26.3): what the owner reads about each.

export const engineering = {
  "software-architect": {
    name: "Software Architect",
    role: "Designs how a system fits together and records the decisions, trade-offs and risks",
    summary: "Designs how the parts of a system fit together and writes down why",
    about:
      "Decides with you how a piece of software should be structured, picks the simplest design that meets your needs and writes down each important decision with its reasons, costs and risks. It designs; it does not write the product's code.",
    when: "Before you build something big, or when a system has become hard to change.",
    pairs:
      "Developer, Frontend, Backend and Mobile Developers, who build from the design, and Security Reviewer.",
  },
  "frontend-developer": {
    name: "Frontend Developer",
    role: "Builds and fixes the part of a site or app that people see and use, fast on a phone and usable by everyone",
    summary: "Builds the screens people use, fast on a phone and usable by everyone",
    about:
      "Builds and fixes pages, forms and menus, thinking about phones first, keyboards, screen readers and slow connections. It checks its work in a real browser and runs the project's tests.",
    when: "When a site or an app needs a new screen, or what exists is slow or confusing.",
    pairs: "Designer, who draws the screens, and Backend Developer, who provides the data.",
  },
  "backend-developer": {
    name: "Backend Developer",
    role: "Builds and fixes the server side: APIs, data handling, background jobs and their tests",
    summary: "Builds the part behind the screen: APIs, data and background jobs, with tests",
    about:
      "Builds and fixes what runs behind the screen: servers, APIs, data rules and jobs, with tests and with attention to what happens when something fails. It never runs changes against live data without asking you.",
    when: "When your product needs to save, process or share data, or connect to another service.",
    pairs: "Database Engineer, for the data, and Frontend Developer, who uses its APIs.",
  },
  "mobile-developer": {
    name: "Mobile Developer",
    role: "Builds and fixes phone apps and checks them on realistic screens, networks and devices",
    summary: "Builds and fixes phone apps, thinking about small screens, slow networks and battery",
    about:
      "Builds and fixes phone apps with small screens, bad networks, interruptions and permissions in mind. It tells you honestly what it could not test without a real device, and it never touches your store accounts or signing keys.",
    when: "When you want an app for phones, or the one you have is slow or crashes.",
    pairs: "Designer, for the screens, Backend Developer, for the API, and QA Tester.",
  },
  "database-engineer": {
    name: "Database Engineer",
    role: "Designs tables, writes and speeds up queries and plans safe changes to a database",
    summary: "Designs your database, makes queries fast and changes data without losing any",
    about:
      "Designs how your data is stored, makes slow queries fast by measuring first and plans every change so that it can be undone, with a backup first. It tests on a copy and never runs anything destructive on live data.",
    when: "When a database is slow, messy, or has to change without losing anything.",
    pairs: "Backend Developer, who applies the changes, and Data Engineer, who loads the data.",
  },
  "devops-engineer": {
    name: "DevOps Engineer",
    role: "Sets up builds, tests, deployments and monitoring, and keeps pipelines and servers healthy",
    summary: "Automates building, testing and shipping, and keeps it all running and watched",
    about:
      "Makes shipping software safe and boring: automatic builds and tests, repeatable deployments, logs and alerts, and a way back for each release. It asks before anything touches a live system and never asks you to paste passwords into the chat.",
    when: "When releasing is manual and scary, or something keeps breaking in production.",
    pairs:
      "Backend Developer, for what the service needs, and Security Reviewer, for access and secrets.",
  },
  "security-reviewer": {
    name: "Security Reviewer",
    role: "Reviews your own code and setup for security weaknesses and explains how to fix them",
    summary: "Looks for security holes in your own code and setup and explains how to fix them",
    about:
      "Reviews your own code and setup for weaknesses, explains each one in plain words with how serious it is and the smallest fix. It works defensively, only on what is yours, and never repeats a secret it finds. It thinks harder than most bots, so it uses a bit more of your plan.",
    when: "Before you launch, after a big change, or when you handle other people's data.",
    pairs: "Developer and DevOps Engineer, who apply the fixes, and Code Reviewer.",
  },
  "data-engineer": {
    name: "Data Engineer",
    role: "Builds reliable pipelines that move, clean and store data so that analysts can trust it",
    summary: "Builds the pipes that bring clean, trustworthy data to where you analyze it",
    about:
      "Builds the paths data takes from where it is created to where it is analyzed: it collects, cleans and checks it at every step and stops when something looks wrong instead of passing bad numbers on. It keeps your originals untouched.",
    when: "When your numbers live in many places or you cannot trust your reports.",
    pairs: "Database Engineer, for where to store it, and Data Analyst, who uses the result.",
  },
};
