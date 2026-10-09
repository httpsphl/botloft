// Game making roles (spec 26.3): what the owner reads about each. All make
// original work and none designs anything meant to trick players into
// spending.

export const games = {
  "game-designer": {
    name: "Game Designer",
    role: "Turns a game idea into clear rules and a loop that is fun to repeat: the goal, the choices, the feedback and a first small version to test",
    summary:
      "Turns your game idea into clear rules and a fun loop, with a small first version to test",
    about:
      "Starts from the experience you want the player to have, finds the loop they repeat and writes the rules in short, exact sentences, checking them for loopholes. It plans the smallest playable version and a test, and when something is not fun it offers two changes to try. The game stays yours.",
    when: "When you have a game idea, digital or on a table, and want it to become something playable.",
    pairs: "Narrative Designer, for the story, and Game Economy Designer, for the numbers.",
  },
  "game-narrative-designer": {
    name: "Narrative Designer",
    role: "Builds the story, characters and world of a game, and writes the text so it fits how the game is played",
    summary:
      "Builds the story, characters and world of your game and writes the text so it fits the play",
    about:
      "Builds the world on one page, makes characters with a want, a flaw and a voice and plans the story around the player's choices and the length of play. It writes the text in short lines with their context, so a developer can place them. Everything is original, and it tells you which age the sensitive themes suit.",
    when: "When your game needs a world, a story, quests or characters that people remember.",
    pairs: "Game Designer, for the rules, and Translator, for other languages.",
  },
  "level-designer": {
    name: "Level Designer",
    role: "Designs the levels, maps or scenarios of a game so that they teach, challenge and surprise in the right order",
    summary:
      "Designs the levels, maps or scenarios of your game so they teach, challenge and surprise in order",
    about:
      "Plans the order in which a player learns each mechanic, one new thing at a time, then describes each level with a map in text: the start, the goal, the route, the challenges and the rewards. It checks for places where a player can get stuck or skip the point, and writes a playtest list for each level.",
    when: "When you have the rules of a game and need the levels or scenarios that use them.",
    pairs: "Game Designer, to check the rules, and Narrative Designer, for the story beats.",
  },
  "game-economy-designer": {
    name: "Game Economy Designer",
    role: "Balances the numbers of a game: costs, rewards, progress and prices, so that it stays fair and interesting, without tricking players into spending",
    summary:
      "Balances the numbers of your game: costs, rewards and progress, fairly and without tricks",
    about:
      "Draws the flows of coins, items and time, puts the numbers in a spreadsheet with formulas and simulates players at different speeds. It finds resources that pile up and paths that are always best. If your game earns money, it proposes fair ways, and never hidden costs, fake countdowns, pay-to-win or paid random rewards aimed at children.",
    when: "When your game has coins, items or progress and the numbers feel off, or you plan to earn money from it.",
    pairs: "Game Designer, for the goals, and Data Analyst, for the simulations.",
  },
  "playtest-analyst": {
    name: "Playtest Analyst",
    role: "Plans playtests and turns what players did and said into clear findings and the changes most worth making",
    summary:
      "Plans playtests and turns what players did and said into clear findings and changes worth making",
    about:
      "Plans the session so you do not explain or defend the game, then turns your notes into findings: the problem, how many players hit it, where and why. It trusts what players did over what they said, ranks the changes by effect for the effort and suggests the next test. It never contacts players.",
    when: "After people have tried your game and you are not sure what to change first.",
    pairs: "Game Designer, who acts on the findings, and Level Designer.",
  },
  "game-audio-director": {
    name: "Game Audio Brief Writer",
    role: "Plans the sound and music of a game and writes clear briefs for whoever makes them: the mood, the list of sounds and how they play",
    summary:
      "Plans the sound and music of your game and writes clear briefs for whoever makes them",
    about:
      "Plans the music and the list of sounds in a table: when each plays, how it should feel and what the player must understand from it. It plans the mix and visual cues for players who cannot hear well, and writes one brief per piece so a composer or sound designer can start. It does not make audio, and reminds you to check rights.",
    when: "When your game is playable and needs sound, or you hire a composer.",
    pairs:
      "Game Designer, for the moments that need sound, and Developer, for how sounds are triggered.",
  },
  "tabletop-game-master": {
    name: "Tabletop Game Master",
    role: "Helps run a tabletop role-playing game: prepares adventures, characters and rules answers, and keeps the story going at the table",
    summary:
      "Helps run a tabletop role-playing game: adventures, characters, rules answers and ideas for the table",
    about:
      "Asks about your system, your group and the tone, then prepares a short adventure with a hook, scenes with a choice each and several endings, plus quick characters. It answers rule questions from the book you have and says when it is not sure. During play it gives three options when you are stuck, and asks the group about limits.",
    when: "When you run or play a role-playing game and want help preparing or improvising.",
    pairs: "Narrative Designer, for deeper lore, and Designer, for maps and handouts.",
  },
};
