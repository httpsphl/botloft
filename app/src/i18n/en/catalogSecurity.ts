// Security roles (spec 26.3): what the owner reads about each. All work
// defensively, on the owner's own things, and never repeat a secret.

export const security = {
  "threat-modeler": {
    name: "Threat Modeler",
    role: "Maps what could go wrong in a system before it is built or changed: what to protect, who might attack it, how, and what to do about it",
    summary:
      "Maps what could go wrong in your system and what to do about it, before it is built or changed",
    about:
      "Draws your system as a short list of parts and flows, marks where trust changes and asks what could go wrong at each place. It ranks the threats by how likely and how bad, in plain words, and proposes defenses with their cost, split into fix before launch and good to have.",
    when: "Before you build or change something that holds valuable data or does something important.",
    pairs: "Security Reviewer, to check the code once it is built, and Software Architect.",
  },
  "incident-responder": {
    name: "Incident Responder",
    role: "Helps you through a security incident: contain it, work out what happened, fix it, tell the right people and learn from it",
    summary:
      "Helps you through a security incident, step by step: contain it, understand it, fix it and learn from it",
    about:
      "When a key leaks, an account is taken over or something looks wrong, it gives you the next step, keeps the timeline and tells you which steps are hard to undo. It asks you to save the evidence, helps you work out how far it went, lists who may need to be told and writes a blameless review at the end. You take the actions.",
    when: "As soon as you suspect something went wrong with security.",
    pairs: "Secrets Auditor, to find other exposed keys, and Backend Developer, to fix the cause.",
  },
  "secrets-auditor": {
    name: "Secrets Auditor",
    role: "Finds passwords, keys and tokens left in your code, files and settings, and plans how to replace and protect them, without repeating their values",
    summary:
      "Finds passwords, keys and tokens left in your code and files and plans how to replace and protect them",
    about:
      "Searches your code, its history, settings and documents for passwords, keys and tokens, and tells you where each one is and what kind it is, never the value. It treats every real find as leaked, gives the order to replace it and plans how to keep secrets out in future. It never uses a secret it finds.",
    when: "Before you make a project public, share it, or hand it over.",
    pairs: "DevOps Engineer, for safe storage, and Security Reviewer.",
  },
  "privacy-engineer": {
    name: "Privacy Engineer",
    role: "Checks what personal data your product collects and keeps, whether it needs it, and how to protect it and respect people's choices",
    summary:
      "Checks what personal data your product collects and keeps, and how to protect it and respect choices",
    about:
      "Maps every kind of personal data you collect, where it goes and how long it stays, and asks whether you need it. It checks who can read it, whether it shows up in logs and whether people can see, correct or delete their data. It lists the laws that may apply as questions for a lawyer and never says something is compliant.",
    when: "Before you launch, collect new data, or when a customer asks how you handle theirs.",
    pairs: "HR Policy Writer, for plain-language notices, and Backend Developer, for the changes.",
  },
  "compliance-checklist": {
    name: "Compliance Checklist Assistant",
    role: "Turns a security or privacy standard into a plain checklist, shows what you already do and what is missing, and gathers the evidence",
    summary:
      "Turns a security or privacy standard into a plain checklist and shows what you do and what is missing",
    about:
      "Takes the standard or the customer questionnaire you have to meet, turns each requirement into a plain question and checks it against what you show it. It marks an item done only with evidence, plans the gaps by effort and effect and drafts honest answers for a questionnaire. It is not an auditor and nothing it writes is a certification.",
    when: "When a customer, a contract or a market asks you to prove your security or privacy.",
    pairs: "Privacy Engineer and Secrets Auditor, for specific findings.",
  },
  "ai-code-auditor": {
    name: "AI-Written Code Auditor",
    role: "Reviews code that an AI wrote for the mistakes it tends to make: invented functions, weak security, missing checks and code nobody understands",
    summary:
      "Reviews code written by an AI for the mistakes it tends to make, before you rely on it",
    about:
      "Checks that the libraries and functions the code uses really exist, looks for the basic security holes this kind of code often skips, tests the edge cases and checks that the tests really test something. It also tells you if the code is more complex than it needs to be. You get a verdict: safe, safe after these fixes, or do not use.",
    when: "Before code that an AI wrote reaches real users or real data.",
    pairs: "Code Reviewer, for a second look, and Security Reviewer.",
  },
  "cloud-config-reviewer": {
    name: "Cloud Setup Reviewer",
    role: "Reviews your cloud and server settings for risky choices: open doors, too much access, missing logs and missing backups",
    summary:
      "Reviews your cloud and server settings for open doors, too much access and missing backups",
    about:
      "Reads the settings you export and finds the usual risks: storage open to the world, accounts with too much access, no second factor, no backups or backups never tested. Each finding comes with the evidence and the smallest safe change, and it warns about changes that could lock you out. It never signs in to your account.",
    when: "After you set up a cloud account or server, and once a year after that.",
    pairs: "DevOps Engineer, who applies the changes, and Secrets Auditor.",
  },
};
