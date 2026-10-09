// Health support roles (spec 26.3): what the owner reads about each. None is
// a doctor and none diagnoses, treats or advises on medicines; each sends
// emergencies to the local emergency number first.

export const health = {
  "appointment-prep-assistant": {
    name: "Appointment Prep Assistant",
    role: "Helps you get the most from a medical visit: a clear history, a symptom diary, your questions in order and a plain note of what was said",
    summary:
      "Helps you get the most from a medical visit: your history, your questions and a plain note afterwards",
    about:
      "Turns what has been happening into a one-page summary the professional can read in a minute, with a timeline, your medicines as written on the packaging and the questions to ask in order of importance. After the visit it helps you write down what was said and nothing more. It does not guess what a symptom means. It is not a doctor.",
    when: "Before any visit to a doctor or other health professional, for you or someone you care for.",
    pairs: "Health Records Organizer, for your papers, and Family Care Organizer.",
  },
  "elder-care-companion": {
    name: "Family Care Organizer",
    role: "Helps a family look after an older or ill relative: appointments, who does what, papers, questions for the care team and the caregiver's own limits",
    summary:
      "Helps a family look after an older or ill relative: appointments, tasks, papers and questions for the care team",
    about:
      "Keeps one calendar of visits and renewals, a weekly plan of who does what and a one-page summary to share with the care team. It lists the medicines exactly as written and never tells you to change one: those questions go to the doctor or the pharmacist. It also looks after you, the caregiver, and helps with hard family conversations.",
    when: "When you are looking after a parent or another relative and the tasks are piling up.",
    pairs: "Appointment Prep Assistant, for each visit, and Medical Bill Helper, for the costs.",
  },
  "clinic-front-desk-assistant": {
    name: "Clinic Front Desk Assistant",
    role: "Helps a small clinic or practice answer patients: opening hours, how to book, preparation for visits and polite replies, without giving medical advice",
    summary:
      "Helps a small clinic answer patients: hours, booking, preparation and polite replies, with no medical advice",
    about:
      "Keeps one sheet with your hours, fees, booking rules and preparation for each visit and drafts replies to patients from it. Anything about symptoms, medicines, results or an urgent matter is set aside for the professional, with a short reply that says someone will answer and to call the emergency number if it is urgent. It never sends a message itself.",
    when: "When patients ask the same practical questions all day.",
    pairs: "Translator, for patients in another language, and Invoicing Assistant, for payments.",
  },
  "medical-bill-helper": {
    name: "Medical Bill Helper",
    role: "Helps you understand medical bills and insurance statements, find mistakes and write a clear appeal or question to the insurer or provider",
    summary:
      "Helps you understand medical bills and insurance statements, find mistakes and write the appeal",
    about:
      "Explains each line of a bill and of the insurer's statement in plain words, in a table, and looks for the usual mistakes, such as a service billed twice or a claim refused for a reason that looks wrong. It drafts the call script or the letter and keeps a log of who said what. It never pays, calls or signs in for you, and does not judge the treatment.",
    when: "When a medical bill or an insurance statement does not make sense, or a claim was refused.",
    pairs: "Bills Assistant, for payment, and Complaint Letter Drafter, for a formal letter.",
  },
  "wellness-habit-coach": {
    name: "Daily Habits Coach",
    role: "Helps you build small, steady habits for sleep, movement, food and stress, at your own pace, without medical claims",
    summary:
      "Helps you build small, steady habits for sleep, movement, food and stress, at your own pace",
    about:
      "Starts from what you want and makes each habit tiny and tied to something you already do, with a plan for the bad days and a simple weekly review. It never shames a miss. It gives no diet plans for an illness, no calorie targets, no supplement advice and no mental health treatment: for those it sends you to the right professional, and if a goal turns harmful it stops coaching it.",
    when: "When you want to sleep better, move more or manage stress, one small step at a time.",
    pairs: "Goals Coach, for longer goals, and Appointment Prep Assistant.",
  },
  "health-evidence-summarizer": {
    name: "Health Evidence Summarizer",
    role: "Finds and summarizes what the research says about a health question, with honest limits and real sources, to bring to a professional",
    summary:
      "Finds and summarizes what research says about a health question, with real sources and honest limits",
    about:
      "Writes your question precisely, looks first for reviews of many studies and recognized guidelines, opens every source before it cites it and explains in plain words how big the effect is, how sure the researchers are, the harms as well as the benefits and what is unknown. It never invents a study. It is general information, not advice for you, and it ends with questions for a professional.",
    when: "When you want to understand what is known about a treatment or a test before you talk to your doctor.",
    pairs: "Fact Checker, to verify a citation, and Editor, to polish.",
  },
  "health-records-organizer": {
    name: "Health Records Organizer",
    role: "Gathers and organizes your health papers into one timeline: visits, tests, medicines and vaccines, without interpreting them",
    summary:
      "Gathers your health papers into one clear timeline: visits, tests, medicines and vaccines",
    about:
      "Gathers your reports, results, prescriptions and vaccine records into one timeline, copying every value exactly as written, and builds a one-page summary to bring to any visit. It flags what is hard to read or inconsistent as a question for the professional. It never says a result is good, bad or normal, and never requests records or signs in to a portal.",
    when: "When your health papers are scattered and you need them in order for a visit.",
    pairs: "Appointment Prep Assistant, who uses the summary for a visit.",
  },
};
