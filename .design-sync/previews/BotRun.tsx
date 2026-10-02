import { BotloftProvider, BotRun, chat, makeBot, makeCrew, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

const crew = makeCrew({ name: "Research" });
const chief = makeBot({ crew, name: "Chief", chief: true, state: "idle" });
const scout = makeBot({ crew, name: "Scout", state: "busy" });
const writer = makeBot({ crew, name: "Writer", state: "needs_approval" });

export function ReplyWithTools() {
  const items = [
    chat.tool(scout, { name: "WebSearch", summary: "on-device AI news this week" }, 3),
    chat.tool(scout, { name: "WebFetch", summary: "https://example.com/llm-on-phones" }, 3),
    chat.tool(scout, { name: "Write", summary: "sources.md", file: "C:\\Users\\owner\\Botloft\\research\\shared\\sources.md" }, 2),
    chat.reply(
      scout,
      "Found **6 sources** this week. The best three are in `sources.md`:\n\n1. A 3B model running fully on a phone\n2. New NPU benchmarks\n3. A survey of on-device RAG\n\nI sent them to @writer.",
      2,
    ),
    chat.turn(scout, 48, 2),
  ];
  return (
    <BotloftProvider crews={[crew]} bots={[chief, scout, writer]}>
      <ul className="bg-canvas p-4" style={{ maxWidth: 720 }}>
        <BotRun items={items} bot={scout} />
      </ul>
    </BotloftProvider>
  );
}

export function AsksForASite() {
  const items = [
    chat.reply(writer, "To check the benchmark numbers I need the paper itself.", 0),
    chat.askSite(writer, "https://arxiv.org/abs/2609.01234"),
  ];
  return (
    <BotloftProvider crews={[crew]} bots={[chief, scout, writer]}>
      <ul className="bg-canvas p-4" style={{ maxWidth: 720 }}>
        <BotRun items={items} bot={writer} live={{ draft: "", working: false }} />
      </ul>
    </BotloftProvider>
  );
}

export function ChiefSuggestsABot() {
  const items = [
    chat.reply(chief, "For a weekly summary the crew needs someone to check facts before it goes out.", 1),
    chat.suggestBot(chief, {
      name: "Editor",
      role: "Checks facts and tone before the summary goes out.",
      model: "sonnet",
      instructions: "Read Writer's draft every Friday morning. Check each number against its source.",
      reason: "Summaries go out without a second look today.",
    }),
  ];
  return (
    <BotloftProvider crews={[crew]} bots={[chief, scout, writer]}>
      <ul className="bg-canvas p-4" style={{ maxWidth: 720 }}>
        <BotRun items={items} bot={chief} live={{ draft: "", working: false }} />
      </ul>
    </BotloftProvider>
  );
}

export function Writing() {
  return (
    <BotloftProvider crews={[crew]} bots={[chief, scout, writer]}>
      <ul className="bg-canvas p-4" style={{ maxWidth: 720 }}>
        <BotRun
          items={[chat.tool(scout, { name: "WebSearch", summary: "NPU benchmarks 2026", status: "running" })]}
          bot={scout}
          live={{ draft: "Two of the new chips run a 3B model at more than", working: true }}
        />
      </ul>
    </BotloftProvider>
  );
}
