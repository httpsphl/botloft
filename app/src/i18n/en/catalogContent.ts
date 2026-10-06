// Content, research and learning roles (spec 26.3): what the owner reads about each.

export const contentAndLearning = {
  editor: {
    name: "Editor",
    role: "Edits and proofreads texts: fixes errors, tightens the writing and keeps the author's voice",
    summary: "Fixes mistakes and tightens your writing while keeping it sounding like you",
    about:
      "Fixes spelling, grammar and clumsy sentences and cuts what is not needed, at the level you ask for: only mistakes, the wording or the structure too. It shows every change with the reason and never changes a fact, a number or a quote.",
    when: "Before you publish or send anything that matters.",
    pairs: "Fact Checker, to verify claims, and Writer, when a part needs rewriting.",
  },
  ghostwriter: {
    name: "Ghostwriter",
    role: "Writes in your voice: posts, speeches, bios and longer pieces that you will publish as your own",
    summary:
      "Writes in your voice what you will publish under your own name, from your ideas and stories",
    about:
      "Learns how you sound, draws out your stories and opinions with questions and writes drafts in your voice. It marks every gap it filled so you can confirm it, never invents experiences, and does not write work you must hand in as your own, like an exam or a thesis.",
    when: "When you have things to say but not the time or the words.",
    pairs: "Researcher, for facts, and Editor, to polish.",
  },
  "fact-checker": {
    name: "Fact Checker",
    role: "Checks claims against reliable sources and says how sure it is, or that it could not tell",
    summary: "Checks whether a claim is true against reliable sources, and says how sure it is",
    about:
      "Takes the claims in a text and checks each one against the best sources it can find, preferring the original document. It gives a plain verdict with the link and how sure it is, and says honestly when it could not tell.",
    when: "Before you publish, and whenever something you read sounds too good to be true.",
    pairs: "Editor and Writer, who apply the corrections.",
  },
  "academic-researcher": {
    name: "Academic Researcher",
    role: "Finds and summarizes scholarly sources on a topic, with real citations and an honest account of the evidence",
    summary:
      "Finds and summarizes scholarly work on a topic, with real citations and an honest view of the evidence",
    about:
      "Finds papers, books and reports on your question, opens each one before citing it and summarizes the finding, the method and the limits. It never invents a reference, says when it only read an abstract, and does not write work you must hand in as your own.",
    when: "When you need to know what the research really says about a topic.",
    pairs: "Data Analyst, for the numbers in a study, and Writer, for a readable summary.",
  },
  tutor: {
    name: "Tutor",
    role: "Teaches a subject step by step, adapting to your level, and checks that you really understood",
    summary:
      "Teaches you a subject step by step, at your level, and checks that you really understood",
    about:
      "Finds out what you already know and teaches one idea at a time with examples from your world. It asks you to explain it back or try a small exercise, gives hints before answers and corrects kindly. It will not do your graded assignment for you.",
    when: "When you want to learn a subject properly, or help someone who is studying.",
    pairs: "Researcher, for sources, and Writer, for study notes.",
  },
  "language-teacher": {
    name: "Language Teacher",
    role: "Teaches a language through conversation, kind corrections, vocabulary and short exercises",
    summary:
      "Helps you learn a language by talking, with kind corrections, vocabulary and short exercises",
    about:
      "Holds conversations at your level in the language you are learning, corrects the mistakes that matter most with the rule in one line, teaches words in context and ends each session with a summary and a small exercise. It works from text, so it cannot hear your pronunciation.",
    when: "When you want to practice a language every day, without shyness.",
    pairs: "Translator, to check a difficult phrase, and Writer, for reading material.",
  },
};
