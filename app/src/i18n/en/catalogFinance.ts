// Finance roles (spec 26.3): what the owner reads about each. None of them
// moves money or replaces an accountant or an adviser, and each says so.

export const finance = {
  bookkeeper: {
    name: "Bookkeeper",
    role: "Keeps your records in order: sorts transactions into categories, matches them to receipts and finds what does not add up",
    summary:
      "Sorts your transactions into categories, matches them to receipts and flags what does not add up",
    about:
      "Takes your statements, receipts and spreadsheets and keeps a clean record: each transaction in a category, matched to its proof, with the totals checked. It lists what has no proof instead of guessing and never changes an original file. It keeps records; it is not an accountant, and it never moves money.",
    when: "At the end of every month, or when your records have piled up.",
    pairs: "Financial Analyst and Budget Planner, who work from clean records.",
  },
  "budget-planner": {
    name: "Budget Planner",
    role: "Builds a realistic budget from your income and spending, tracks it through the month and shows where the money goes",
    summary:
      "Builds a realistic budget from your income and spending, and shows where the money goes",
    about:
      "Starts from your real numbers and builds a simple plan: what comes in, what goes out and what you want to reach. If the plan does not close, it says so first and shows a few ways to close it. During the month it shows where you are ahead of plan. It does not advise on investments or loans.",
    when: "When you want to know where the money goes, or to plan a goal.",
    pairs: "Bookkeeper, for clean records, and Cash Flow Forecaster, for what comes next.",
  },
  "financial-analyst": {
    name: "Financial Analyst",
    role: "Reads financial statements and reports, calculates the key measures and explains what they say about the business",
    summary:
      "Reads your financial statements, calculates the key measures and explains what they mean",
    about:
      "Reads your statements and spreadsheets, checks that the numbers add up and calculates what matters for your question: margins, growth, break-even, return on a project. It shows every formula, compares with something meaningful and says what the numbers do not tell. It does not tell you what to buy, sell or invest in.",
    when: "Before a decision that depends on the numbers, or to understand how the business is doing.",
    pairs: "Bookkeeper, for the records, and Data Analyst, for a deeper cut of the data.",
  },
  "cash-flow-forecaster": {
    name: "Cash Flow Forecaster",
    role: "Projects the money coming in and going out week by week and warns early when cash could run short",
    summary:
      "Projects the money in and out week by week and warns you early if cash could run short",
    about:
      "Builds a forecast from today's balance, the bills due and the money you expect, and shows the lowest point and when it comes. It gives an expected, a cautious and a good case and labels every estimate. If cash could run short, it lists your options and what each costs. It never moves money or delays a payment itself.",
    when: "When money is tight, a big bill is coming, or you are about to commit to a cost.",
    pairs: "Bills Assistant, for what is due, and Invoicing Assistant, for what is owed to you.",
  },
  "bills-assistant": {
    name: "Bills Assistant",
    role: "Keeps track of what you owe: lists the bills and due dates, checks them against orders and contracts, and prepares each payment for you to make",
    summary:
      "Tracks your bills and due dates, checks them for mistakes and prepares each payment for you to make",
    about:
      "Collects your bills, lists the amount and due date of each, and checks them against the order or contract for duplicates, wrong totals and charges nobody agreed. It keeps one list sorted by due date and warns when a bill asks for urgent payment to a new account, a common fraud. You make every payment yourself.",
    when: "When bills arrive from many places and you do not want to pay late, or twice.",
    pairs: "Bookkeeper, for the paid ones, and Cash Flow Forecaster, for what is coming.",
  },
  "invoicing-assistant": {
    name: "Invoicing Assistant",
    role: "Prepares your invoices and payment reminders, and keeps track of what customers still owe you",
    summary:
      "Prepares your invoices and polite payment reminders, and tracks what customers still owe you",
    about:
      "Prepares invoices from the work you did, numbers them in order and keeps a list of what is sent, paid and late. For a late invoice it writes a reminder in three steps, from friendly to firm, never threatening. It marks the legal fields it cannot fill for you or your accountant, and you send everything yourself.",
    when: "When you bill customers and chasing payments takes too much of your time.",
    pairs: "Bookkeeper, for the paid ones, and Cash Flow Forecaster, for the due dates.",
  },
  "tax-organizer": {
    name: "Tax Organizer",
    role: "Gathers and organizes the documents and numbers for your taxes through the year, so your accountant or your filing starts ready",
    summary:
      "Gathers and organizes your tax papers and numbers through the year so filing starts ready",
    about:
      "Collects and sorts the papers you need for taxes, keeps one list of what is complete and what is missing, and warns you before the dates that matter. It does not decide what is deductible: it lists the doubtful items as questions for your accountant. It never files, pays or signs in to a tax site, and it is not an accountant.",
    when: "Through the year, and in the weeks before you or your accountant file.",
    pairs: "Bookkeeper, for the records, and Invoicing Assistant, for the income side.",
  },
};
