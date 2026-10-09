// Legal roles (spec 26.3): what the owner reads about each. None is a
// lawyer; each says so and sends the owner to one for anything that matters.

export const legal = {
  "contract-reader": {
    name: "Contract Reader",
    role: "Reads a contract in plain words: what each side must do, the dates and money, the risky clauses and the questions to ask a lawyer",
    summary: "Reads a contract in plain words and flags the risky clauses and what to ask a lawyer",
    about:
      "Reads the whole contract and gives you one page: who must do what, the money, the dates and how it ends. It flags the clauses that deserve attention, such as automatic renewal, penalties or giving up rights, and marks each as common, unusual or worth negotiating. It never says a clause is valid or enforceable. It is not a lawyer, and it ends with the questions to take to one.",
    when: "Before you sign a contract, or when you want to understand one you already signed.",
    pairs: "Contract Dates Tracker, for the dates, and Legal Research Assistant.",
  },
  "terms-drafter": {
    name: "Terms and Privacy Notice Drafter",
    role: "Drafts the terms of use and the privacy notice of a site or app in plain words, from how the business really works, for a lawyer to check",
    summary:
      "Drafts your site's terms and privacy notice in plain words from how you really work, for a lawyer to check",
    about:
      "Asks how your business really works and what data it collects, then drafts the terms of use, the privacy notice and the refund and shipping policy in short, plain sentences. The parts that depend on the law are marked as questions for a lawyer, and it lists the promises you will have to keep. It never publishes anything on your site.",
    when: "When you launch a site, an app or a shop and need the texts that go with it.",
    pairs: "Privacy Engineer, for the data map, and Editor, to polish.",
  },
  "legal-research-assistant": {
    name: "Legal Research Assistant",
    role: "Finds and explains the laws, rules and decisions on a question from official sources, with exact citations, and says what it could not verify",
    summary:
      "Finds the laws and decisions on a question from official sources, with exact citations and honest limits",
    about:
      "Needs the place, the date and the facts, then looks in official sources first, opens every source before it cites it and gives you a short memo with exact references. It separates what the law says from what courts say about it and what is unsettled. It never invents a law or a case, and says when it could not verify one.",
    when: "When you need to know what the law says on a question, before you talk to a lawyer.",
    pairs: "Fact Checker, to verify a citation, and Editor, to polish the memo.",
  },
  "legal-intake-assistant": {
    name: "Client Intake Assistant",
    role: "Helps a small law practice or advisor collect a new client's facts: a clear questionnaire, a neat summary of the matter and the missing documents",
    summary:
      "Helps a small practice collect a new client's facts: a clear questionnaire, a neat summary and what is missing",
    about:
      "Prepares a plain intake form for the kind of matter, then turns the client's answers into a timeline and a one-page summary, with facts kept apart from opinions. It lists the missing documents and puts anything urgent on top. It never tells a client their rights or whether they have a case, and never contacts a client.",
    when: "At the first step with a new client, before the lawyer reviews the matter.",
    pairs: "Time and Billing Assistant, and Contract Reader.",
  },
  "legal-billing-assistant": {
    name: "Time and Billing Assistant",
    role: "Turns your work log into clear time entries and invoice drafts for a practice or consultant, and shows what is unbilled",
    summary:
      "Turns your work log into clear time entries and invoice drafts, and shows what is still unbilled",
    about:
      "Turns your notes and calendar into time entries with descriptions a client would understand and that do not reveal more than they need to. It shows what is unbilled by client, flags entries that look wrong without changing them and prepares the invoice draft from the entries you approve. It never rounds up or overstates the work.",
    when: "When you bill by the hour and the time slips away before you invoice.",
    pairs: "Invoicing Assistant, for payment follow-up, and Bookkeeper.",
  },
  "contract-deadline-tracker": {
    name: "Contract Dates Tracker",
    role: "Keeps track of the dates in your contracts: renewals, notice periods, payments and deadlines, and warns you well before each one",
    summary:
      "Keeps track of the dates in your contracts, such as renewals and notice periods, and warns you early",
    about:
      "Reads your contracts and lists every date and duty with the clause it comes from, working out the real date and saying in plain words how it was counted. It puts the traps first, such as automatic renewals with a notice period, and warns you early. If a clause is unclear it says so instead of guessing. It never sends a notice or ends a contract.",
    when: "When you have several contracts and are afraid of missing a renewal or a notice.",
    pairs: "Contract Reader, to explain a clause, and Writer, to draft a notice for you to send.",
  },
  "complaint-letter-drafter": {
    name: "Complaint Letter Drafter",
    role: "Drafts clear, firm and polite letters for a complaint or a dispute with a company or a landlord, from the facts and the papers you have",
    summary:
      "Drafts clear, firm and polite complaint letters for a dispute with a company or a landlord",
    about:
      "Gets the facts straight with you, builds a timeline checked against your papers and drafts a short, polite and firm letter: what happened, what you ask for and a reasonable date to reply. It does not threaten or exaggerate. It lists the attachments and the places to check next as questions for your country. You send the letter.",
    when: "When a company, a landlord or a seller has not done right by you.",
    pairs: "Contract Reader, for the terms, and Editor, to proofread.",
  },
};
