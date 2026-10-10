// Design roles (spec 26.3): what the owner reads about each.

export const design = {
  "brand-guardian": {
    name: "Brand Guardian",
    role: "Keeps everything on brand: writes a short brand guide from your material and checks pieces against it",
    summary: "Writes a short brand guide from your material and checks that everything follows it",
    about:
      "Gathers your logo, colors, fonts and examples into a one-page brand guide, marking what it saw and what it suggests. Then it checks pages, posts and decks against the guide and lists exactly where each one drifts. It never invents a brand fact and never changes a rule without asking you.",
    when: "When you have a brand and several people or agents make things for it.",
    pairs: "Designer, who applies the visual fixes, and Writer, who applies the voice fixes.",
  },
  "ui-designer": {
    name: "UI Designer",
    role: "Builds the visual system of an interface: colors, type, spacing and components, shown on a style page",
    summary:
      "Builds the colors, type, spacing and buttons of an interface and shows them on one style page",
    about:
      "Chooses a small palette, a type scale and a spacing step, then builds the buttons, fields, cards and messages with all their states on one style page you can see. Every later screen reuses those parts, so the product looks like one thing.",
    when: "At the start of a product, or when your screens stopped looking like they belong together.",
    pairs:
      "Designer, who makes the pages, Frontend Developer, who builds them, and Accessibility Reviewer.",
  },
  "ux-architect": {
    name: "UX Architect",
    role: "Plans how a product is organized and how people move through it: site maps, user flows and plain wireframes",
    summary:
      "Plans how a product is organized and how people move through it, with flows and simple wireframes",
    about:
      "Starts from who the people are and what they want to do, groups the content the way they think and draws each flow, including errors, empty states and going back. The wireframes are plain grey screens, so you judge the structure before any decoration. It lists the places people are likely to get lost.",
    when: "Before you design or build a product, a site or a big new feature.",
    pairs: "UX Researcher, to test the guesses, and UI Designer, who gives the wireframes a look.",
  },
  "accessibility-reviewer": {
    name: "Accessibility Reviewer",
    role: "Checks pages and designs so people with different abilities can use them, and explains each fix in plain words",
    summary:
      "Checks that your pages work for people with different abilities and explains each fix plainly",
    about:
      "Reads the code of a page and checks what can be read there: contrast, image descriptions, form labels, headings, keyboard use, visible focus and information carried by color alone. Each problem comes with who it affects and the exact fix. It says what it cannot check, such as a real screen reader, and never calls a page certified.",
    when: "Before you launch a page, and after a big change to one.",
    pairs:
      "Frontend Developer, who applies the fixes, and UI Designer, who plans for it from the start.",
  },
  "presentation-designer": {
    name: "Presentation Designer",
    role: "Turns your message into a clear slide deck: the story, one idea per slide and a clean look, shown as screens",
    summary:
      "Turns what you want to say into a clear slide deck, one idea per slide, that you can see take shape",
    about:
      "Writes the story first, then designs each slide as a screen you see appear in the design area: one idea, a title that states the point, large text and one picture or chart at most. The details go into speaker notes. The slides are screens, not a PowerPoint file.",
    when: "When you have to present, pitch or teach something and want it clear.",
    pairs: "Researcher, for facts, Writer, for the words, and Infographic Designer, for charts.",
  },
  "infographic-designer": {
    name: "Infographic Designer",
    role: "Turns numbers and ideas into clear charts and infographics, drawn as screens, without bending the data",
    summary: "Turns numbers and ideas into clear charts and infographics, without bending the data",
    about:
      "Checks where the numbers come from, then picks the form that makes the message clear and draws it as a screen: honest scales, direct labels, few colors, the source and the date on the picture. It tells you what the picture does not show and never invents a number to fill a gap.",
    when: "When a table or a report is hard to read and a picture would do it better.",
    pairs: "Data Analyst, for the numbers, and Presentation Designer, to put it in a deck.",
  },
  "image-prompt-writer": {
    name: "Image Prompt Writer",
    role: "Writes clear, consistent descriptions that you paste into an image or video generator, and refines them from the results",
    summary:
      "Writes clear, consistent descriptions for the image or video generator you use, and refines them",
    about:
      "Does not make images itself: it writes the descriptions you paste into the tool you use, with the subject, setting, style, light and framing, in two or three variations. For a set that must look alike, it writes one shared style paragraph. Show it what came out and it changes only the part that caused the problem.",
    when: "When you use an image or video generator and your results are random or inconsistent.",
    pairs: "Brand Guardian, to keep the style on brand, and Social Media, for the posts.",
  },
  "design-critic": {
    name: "Design Critic",
    role: "Reviews screens and pages with a fresh eye and gives a short, ordered list of what to improve, and why",
    summary:
      "Looks at your screens with a fresh eye and tells you, in order, what to improve and why",
    about:
      "Reads the goal and the screen files and tells you the three things that matter most first: what the eye sees first, whether the main action is clear, spacing, type, contrast and how it works on a small screen. It separates clear problems from matters of taste, says what to keep and does not redesign unless you ask.",
    when: "When a screen feels off and you cannot say why, or before you ship.",
    pairs: "Designer, who makes the changes, and Accessibility Reviewer, for access.",
  },
};
