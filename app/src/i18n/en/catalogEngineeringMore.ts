// More engineering roles (spec 26.3): what the owner reads about each. All
// work on the owner's own code, with the Manual mode's approvals.

export const engineeringMore = {
  "technical-writer": {
    name: "Technical Writer",
    role: "Writes clear documentation from the real code and setup: guides, references and how-tos that match what the system does",
    summary:
      "Writes clear docs from your real code and setup: guides, how-tos and references that stay true",
    about:
      "Reads your code and runs the commands it documents on a copy, so the steps and the output are real. It writes for one reader at a time: the most common task first, numbered steps, what you should see after each one and what to do when it fails. It tells you what it tested and what it only read.",
    when: "When your project has no README, the docs are out of date, or someone new has to take over.",
    pairs: "Codebase Guide, for the big picture, and Code Reviewer, to check the technical claims.",
  },
  "sre-engineer": {
    name: "Reliability Engineer",
    role: "Makes your service dependable: defines what good looks like, sets up alerts that matter and writes the steps to follow when it breaks",
    summary:
      "Makes your service dependable: clear targets, alerts that matter and steps to follow when it breaks",
    about:
      "Turns what your users feel into a few targets in numbers, maps where the service can fail and proposes alerts that mean a person must act now, each with the first thing to check. It writes short runbooks for the usual failures and a blameless review after an incident. It never restarts or changes anything in production on its own.",
    when: "When your service goes down too often, or you get alerts you ignore.",
    pairs: "DevOps Engineer, for the infrastructure, and Backend Developer, for the fixes in code.",
  },
  "release-engineer": {
    name: "Release Engineer",
    role: "Plans and prepares releases: version numbers, change notes, checks before shipping and a way back, without publishing for you",
    summary:
      "Plans and prepares your releases, with notes, a checklist and a way back, and never ships by itself",
    about:
      "Reads what changed since the last release, groups it by what users notice and proposes the version number and the notes in plain words. It writes a checklist, the steps to ship and the steps to undo, and says which one is hard to undo. You make the tag, the upload and the announcement yourself, and it never touches your signing keys.",
    when: "Before every release of an app, a library or a site.",
    pairs: "QA Tester, to run the checklist, and DevOps Engineer, for the pipeline.",
  },
  "codebase-guide": {
    name: "Codebase Guide",
    role: "Explains an unfamiliar codebase: the big picture, where things live, how a request flows and where to start changing",
    summary:
      "Explains an unfamiliar codebase: the big picture, where things live and where to start",
    about:
      "Tailors a tour to what you need to do, traces one real path from start to end by file and function and points out the conventions, the risky areas and the tests. It says what it is sure of and what it inferred, and ends with a small first task. It only reads; it changes nothing unless you ask.",
    when: "When you inherit code, join a project or have to change something you did not write.",
    pairs: "Technical Writer, to turn the tour into docs, and Software Architect.",
  },
  "prompt-engineer": {
    name: "Instruction Writer for AI",
    role: "Writes and improves the instructions you give to AI models, then tests them on real examples before you rely on them",
    summary:
      "Writes and improves instructions for AI models and tests them on real examples before you trust them",
    about:
      "Asks for real examples, including hard ones, writes the instruction with a clear goal, rules, answer format and examples, and tests it, changing one thing at a time. You get a table of results for each version and the weak spots that remain. It warns you when the instruction will read untrusted text that could give orders.",
    when: "When an AI feature in your app or workflow gives answers you cannot rely on.",
    pairs: "Backend Developer, to wire it in, and QA Tester, to try more cases.",
  },
  "rapid-prototyper": {
    name: "Prototype Builder",
    role: "Builds a quick working version of an idea to test it with real people, in the simplest way that works, and says what is faked",
    summary:
      "Builds a quick working version of your idea to test with real people, and says what is faked",
    about:
      "Starts from the question the prototype must answer and builds only that, with made-up data and no real accounts. It writes a short test for the people who will try it and a note saying what is faked and what a real version needs, so nobody ships the prototype by accident.",
    when: "When you have an idea and want to know if people care before you build it properly.",
    pairs: "Designer, for the look, and UX Researcher, to run the test.",
  },
  "localization-engineer": {
    name: "Localization Engineer",
    role: "Prepares software for several languages and countries: moves text out of the code, handles dates, numbers and plurals, and checks the result",
    summary:
      "Prepares your software for several languages and countries and checks that nothing breaks",
    about:
      "Moves the text out of your code into translation files, with one sentence per key and placeholders instead of joined pieces. It handles plurals, dates, numbers and currencies by the user's region, checks layouts with long words and adds a test that fails when a language is missing a string. It hands the new strings to a translator.",
    when: "When you want your app or site in a second language, or a country with different formats.",
    pairs: "Translator, for the strings, and Frontend Developer, for the layout.",
  },
  "refactoring-engineer": {
    name: "Code Cleanup Engineer",
    role: "Cleans up code in small, safe steps without changing what it does, proving each step with tests",
    summary:
      "Cleans up your code in small, safe steps without changing what it does, with tests for each step",
    about:
      "Starts from a reason, makes sure tests exist before it touches anything and then changes one thing at a time, each as its own small commit. It does not mix a cleanup with a behavior change, and it never edits a test just to make a cleanup pass. It lists what it saw but left alone.",
    when: "When code is hard to change or understand and you want it better without breaking it.",
    pairs: "Code Reviewer, to check each step, and QA Tester, to try the behavior.",
  },
};
