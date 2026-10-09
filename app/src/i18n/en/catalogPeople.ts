// People & HR roles (spec 26.3): what the owner reads about each. None
// decides anything about a person; each says so.

export const people = {
  "training-designer": {
    name: "Training Designer",
    role: "Designs training for a team: what people must learn, the lessons and exercises, and how to tell it worked",
    summary:
      "Designs training for your team, with lessons, exercises and a way to tell if it worked",
    about:
      "Starts from what people should do differently afterwards, not from the topic. Writes goals you can observe, short lessons with a practice for each, exercises with answers and notes for whoever teaches. It also says how you will know it worked a month later, and tells you if training is not what is missing.",
    when: "When a team has to learn a new skill, tool or way of working, or a new hire has to get up to speed.",
    pairs: "Change Manager, for rolling it out, and Presentation Designer, for the slides.",
  },
  "change-manager": {
    name: "Change Manager",
    role: "Plans how to bring a change to a team: who it affects, what to say and when, how to hear concerns and how to see if it took hold",
    summary:
      "Plans how to bring a change to your team: who it touches, what to say, when, and how to hear concerns",
    about:
      "Takes a change, such as a new tool or a new process, and plans how to bring it to the people it affects: who hears first and in what words, the questions to expect with honest answers, how concerns come back to you and the support afterwards. It does not announce anything itself, and it never spins the hard parts.",
    when: "Before you introduce something that will change how people work.",
    pairs: "Training Designer, for the learning, and Writer, for the messages.",
  },
  "performance-review-coach": {
    name: "Performance Review Coach",
    role: "Helps managers prepare fair, specific feedback and review conversations, from the facts and examples they bring",
    summary:
      "Helps you prepare fair, specific feedback and review conversations from the facts you bring",
    about:
      "Works from the examples you bring and writes feedback that is specific, about the work and balanced, then checks its own words for bias. It plans the conversation and sets goals you can check. Pay, promotion and warnings stay your decisions, and it never contacts the person.",
    when: "Before a review, a hard conversation or a round of feedback.",
    pairs: "Writer, to polish the wording, and Goals Coach, for next-period goals.",
  },
  "hr-policy-writer": {
    name: "HR Policy Writer",
    role: "Drafts clear workplace policies and an employee handbook in plain words, for a lawyer or advisor to check",
    summary:
      "Drafts clear workplace policies and a staff handbook in plain words, for a professional to check",
    about:
      "Drafts policies on time off, remote work, expenses, conduct and complaints, each with its purpose, rule and steps, plus a two-minute version. It does not state the law: passages that depend on labor law are marked as questions for a lawyer or an HR advisor in your country. Nothing it writes is legally checked.",
    when: "When your team has grown and the rules live only in people's heads.",
    pairs: "Editor, to polish, and Change Manager, to roll the policies out.",
  },
  "engagement-survey-analyst": {
    name: "Engagement Survey Analyst",
    role: "Designs short staff surveys and reads the answers, protecting anonymity, and turns them into a few clear actions",
    summary:
      "Designs short staff surveys, reads the answers while keeping people anonymous, and finds the actions",
    about:
      "Asks first whether you will act on the answers, then writes a short, neutral survey and says plainly who will see the results. It reads the answers by theme and never shows a result for a group so small that a person could be recognized. It ends with three to five actions and a message to the team.",
    when: "When you want to know how your team really feels, and are ready to do something about it.",
    pairs:
      "Data Analyst, for a large set of answers, and Writer, for the message back to the team.",
  },
  "resume-tailor": {
    name: "Resume Tailor",
    role: "Rewrites your resume for a specific job, from your real experience, and prepares you for the interview",
    summary:
      "Rewrites your resume for a specific job from your real experience, and prepares you for the interview",
    about:
      "Reads the job post, matches each requirement to something you truly did and rewrites your resume and a short cover letter for that job. It asks for numbers and examples, prepares the likely interview questions with answers built from your own stories, and never invents a job, a degree, a skill or a number.",
    when: "Whenever you apply for a job that matters.",
    pairs: "Career Coach, for the bigger picture, and Editor, to proof the final texts.",
  },
  "career-coach": {
    name: "Career Coach",
    role: "Helps you think through your next step at work: what you want, what you are good at, the options and a small plan",
    summary: "Helps you think through your next step at work and turn it into a small, doable plan",
    about:
      "Listens first, then helps you see what you want, what you are good at and two or three real options with what each one costs. It turns your choice into three small steps for the next month and helps you prepare a hard conversation, such as asking for a raise. It is not a therapist or an adviser, and the choice is yours.",
    when: "When you are unsure about your next step at work, or facing a hard choice.",
    pairs:
      "Resume Tailor, for the application, and Researcher, for facts about a field or an employer.",
  },
};
