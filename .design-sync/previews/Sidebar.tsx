import { BotloftProvider, makeBot, makeCrew, setLocaleChoice, Sidebar } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

const research = makeCrew({ name: "Research" });
const launch = makeCrew({ name: "Launch" });
const bots = [
  makeBot({
    crew: research,
    name: "Chief",
    chief: true,
    state: "idle",
    activity: { kind: "reply", text: "Scout and Writer are on it. Summary on Friday.", minutesAgo: 12 },
  }),
  makeBot({
    crew: research,
    name: "Scout",
    state: "busy",
    activity: { kind: "tool", text: "on-device AI benchmarks 2026", tool: "WebSearch", minutesAgo: 1 },
  }),
  makeBot({
    crew: research,
    name: "Writer",
    state: "needs_approval",
    activity: { kind: "approval", text: "arxiv.org", tool: "mcp__botloft__browser", minutesAgo: 0 },
  }),
  makeBot({
    crew: launch,
    name: "Designer",
    chief: true,
    state: "idle",
    activity: { kind: "reply", text: "The landing page is ready for a look.", minutesAgo: 40 },
    repliedMinutesAgo: 40,
  }),
];

export function TwoCrews() {
  return (
    <BotloftProvider crews={[research, launch]} bots={bots} selectedBotId={bots[1]?.id}>
      <div className="flex bg-canvas" style={{ height: 600 }}>
        <Sidebar />
      </div>
    </BotloftProvider>
  );
}
