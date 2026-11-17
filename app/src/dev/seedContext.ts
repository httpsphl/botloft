// The preview's effort, conversation space and background tasks (spec 7.4,
// 8.6, 33.4): what Claude Code would have reported for each bot, and a
// compaction that ends a moment after it is asked for.

import type { FakeBotloft } from "../lib/fake";
import type { BotId, ContextPart, ContextUsage } from "../lib/protocol.gen";

const COMPACTION_MS = 2500;

/** What fills a conversation besides the talk, as Claude Code 2.1.309 split it (spec 33.3). */
const FIXED: ContextPart[] = [
  { kind: "instructions", tokens: 3_341 },
  { kind: "tools", tokens: 13_908 },
  { kind: "connected_tools", tokens: 2_500 },
  { kind: "memory", tokens: 1_240 },
  { kind: "skills", tokens: 2_210 },
];

const holds = (usedTokens: number, windowTokens: number): ContextUsage => {
  const fixed = FIXED.reduce((sum, part) => sum + part.tokens, 0);
  return {
    usedTokens,
    windowTokens,
    autoCompactTokens: windowTokens - 33_000,
    compacting: false,
    updatedAt: Date.now(),
    parts: [
      { kind: "messages", tokens: Math.max(0, usedTokens - fixed) },
      ...FIXED,
      { kind: "buffer", tokens: 33_000 },
    ],
    memoryFiles: [
      { path: "C:\\Users\\owner\\Botloft\\research\\scout\\CLAUDE.md", tokens: 4_120, lines: 248 },
      { path: "C:\\Users\\owner\\Botloft\\research\\shared\\CLAUDE.md", tokens: 300, lines: 18 },
    ],
  };
};

export function seedContext(
  fake: FakeBotloft,
  bots: { scout: BotId; writer: BotId; reviewer: BotId; analyst: BotId; planner: BotId },
): void {
  for (const id of [bots.scout, bots.writer, bots.reviewer, bots.planner]) {
    fake.bot(id).effortDefault = "medium";
  }
  // Haiku takes no effort level.
  fake.bot(bots.analyst).effortDefault = "none";
  fake.bot(bots.reviewer).effort = "high";
  fake.setContext(bots.scout, holds(556_000, 1_000_000));
  fake.setContext(bots.writer, holds(912_400, 1_000_000));
  fake.setContext(bots.reviewer, holds(61_800, 1_000_000));
  fake.setContext(bots.analyst, holds(24_300, 200_000));
  const now = Date.now();
  fake.setBackground(bots.scout, [
    {
      taskId: "bn5jw1at6",
      kind: "command",
      description: "Serve the site preview on port 4321",
      seenAt: now - 12 * 60_000,
    },
    {
      taskId: "a5c2e8f41",
      kind: "agent",
      description: "Check every link in the draft",
      seenAt: now - 3 * 60_000,
    },
  ]);

  // The owner's claude.ai account, with Google Calendar given to Scout (spec 34).
  fake.changes.connectors = {
    connectors: [
      { name: "Gmail", state: "connected" },
      { name: "Google Calendar", state: "connected" },
      { name: "Google Drive", state: "connected" },
      { name: "Figma", state: "needs_auth" },
    ],
    checkedAt: now,
  };
  fake.bot(bots.scout).connectors = ["Google Calendar"];
  // Sites the owner signed in to for the agents (spec 35).
  fake.logins.add("reddit.com", [bots.scout], now - 2 * 3_600_000);
  fake.logins.add("notion.so", [], now - 26 * 3_600_000);
  // The owner's own skills, one given to Scout (spec 38).
  fake.skills.add({
    name: "deploy",
    description: "Builds the site and ships it to the server, checking the build first.",
    allowedTools: ["Bash(npm run build *)"],
    hasFiles: true,
    botIds: [bots.scout],
  });
  fake.skills.add({
    name: "brand-voice",
    description: "Writes in the bakery's voice: warm, short, no jargon.",
  });
  fake.skills.add({
    name: "pdf",
    description: "Reads and fills in PDF files.",
    source: "claude_ai",
  });
  // Enough of them for the page's search (spec 38.4).
  fake.skills.add({ name: "humanizer", description: "Rewrites text so it reads like a person." });
  fake.skills.add({ name: "invoice", description: "Makes an invoice from an order." });
  fake.skills.add({
    name: "web-qa",
    description: "Checks a site in the browser for broken pages.",
    // Its downloaded packages put it past the copy limit (spec 38.3).
    tooBig: true,
  });
  fake.skills.add({ name: "weekly-report", description: "Sums up the week's sales." });
  // What Scout can run by typing "/" (spec 38.11).
  fake.skills.slash.set(bots.scout, [
    { name: "code-review", description: "", from: "claude_code" },
    { name: "dataviz", description: "", from: "claude_code" },
    {
      name: "deploy",
      description: "Builds the site and ships it to the server, checking the build first.",
      from: "owner",
    },
    { name: "humanizer", description: "Rewrites text so it reads like a person.", from: "owner" },
    { name: "vercel:deploy", description: "Deploys the site to Vercel.", from: "plugin" },
  ]);
  // The plugins on in the owner's Claude Code, one given to Scout (spec 41).
  fake.plugins.add({
    name: "vercel",
    description: "Build and deploy web apps and agents.",
    version: "0.50.0",
    skills: ["vercel:bootstrap", "vercel:deploy", "vercel:env", "vercel:nextjs"],
    agents: ["vercel:deployment-expert"],
    hooks: ["PostToolUse", "SessionEnd", "SessionStart"],
    mcpServers: ["vercel"],
    botIds: [bots.scout],
  });
  fake.plugins.add({
    name: "data",
    description: "Write SQL, explore datasets, and turn raw data into clear stories.",
    version: "1.1.0",
    source: "claude_ai",
    skills: ["data:analyze", "data:sql-queries", "data:write-query"],
    mcpServers: ["bigquery", "hex"],
  });
  fake.plugins.add({
    name: "frontend-design",
    description: "Distinctive, intentional visual design for new interfaces.",
    version: "1.1.1",
    skills: ["frontend-design:frontend-design"],
  });
  fake.changes.byBot.set(bots.scout, {
    repos: [
      {
        path: "D:\\Projects\\Bakery site",
        name: "Bakery site",
        branch: "feat/menu-page",
        ahead: 2,
        behind: 0,
        files: [
          { path: "src/pages/menu.astro", status: "modified", added: 48, removed: 6 },
          { path: "src/styles/menu.css", status: "added", added: 112, removed: 0 },
          { path: "public/photos/croissant.jpg", status: "untracked", added: null, removed: null },
        ],
        more: 0,
      },
    ],
    prs: [
      {
        number: 14,
        url: "https://github.com/owner/bakery-site/pull/14",
        title: "Menu page with photos and prices",
        state: "open",
        additions: 160,
        deletions: 6,
        checks: { passed: 3, failed: 0, pending: 1 },
      },
    ],
    github: "ok",
    events: [
      {
        kind: "pr",
        sha: null,
        branch: null,
        action: "created",
        number: 14,
        url: "https://github.com/owner/bakery-site/pull/14",
        at: now - 2 * 60_000,
      },
      {
        kind: "push",
        sha: null,
        branch: "feat/menu-page",
        action: null,
        number: null,
        url: null,
        at: now - 3 * 60_000,
      },
      {
        kind: "commit",
        sha: "4f9a2c1",
        branch: "feat/menu-page",
        action: "committed",
        number: null,
        url: null,
        at: now - 4 * 60_000,
      },
    ],
  });

  fake.subscribe((event) => {
    if (event.name !== "bot.context" || !event.params.context?.compacting) {
      return;
    }
    const { botId, context } = event.params;
    setTimeout(() => {
      fake.chat.add(botId, {
        kind: "notice",
        level: "info",
        code: "compacted",
        text: "The conversation was compacted: earlier messages are now a summary.",
      });
      fake.setContext(botId, holds(24_000, context.windowTokens));
    }, COMPACTION_MS);
  });
}
