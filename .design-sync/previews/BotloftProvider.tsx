import {
  BotloftProvider,
  BotRun,
  BotStateBadge,
  chat,
  InboundRow,
  makeBot,
  makeCrew,
  makeMessage,
  setLocaleChoice,
  Sidebar,
} from "@botloft/ui";
import { useState } from "react";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

const crew = makeCrew({ name: "Research" });
const chief = makeBot({ crew, name: "Chief", chief: true, state: "idle" });
const scout = makeBot({
  crew,
  name: "Scout",
  state: "needs_approval",
  activity: { kind: "approval", text: "arxiv.org", tool: "mcp__botloft__browser", minutesAgo: 0 },
});
const writer = makeBot({ crew, name: "Writer", state: "idle" });
const site = chat.askSite(scout, "https://arxiv.org/abs/2609.01234");

// The app's layout, put together: sidebar on the left, the open bot's chat
// on the right. Allow on the card moves Scout back to work.
export function AppScreen() {
  const [allowed, setAllowed] = useState(false);
  const scoutNow = allowed ? { ...scout, state: "busy" as const } : scout;
  const card = allowed ? { ...site, body: { ...site.body, status: "allowed" as const } } : site;
  return (
    <BotloftProvider
      crews={[crew]}
      bots={[chief, scoutNow, writer]}
      selectedBotId={scout.id}
      onAnswer={(answer) => setAllowed(answer.allow)}
    >
      <div className="flex border border-line bg-canvas" style={{ height: 520 }}>
        <Sidebar />
        <main className="flex min-w-0 flex-1 flex-col">
          <header className="flex h-12 items-center gap-3 border-line border-b px-5">
            <span className="font-semibold">Scout</span>
            <BotStateBadge bot={scoutNow} compact />
          </header>
          <ul className="flex flex-col gap-5 overflow-y-auto p-5">
            <InboundRow
              bot={scout}
              message={makeMessage({ from: chief, to: scout, kind: "task", body: "Find this week's best sources on on-device AI.", minutesAgo: 6 })}
            />
            <BotRun
              items={[chat.tool(scout, { name: "WebSearch", summary: "on-device AI news this week" }, 4), chat.reply(scout, "One paper looks key. I need to open it.", 1), card]}
              bot={scoutNow}
              live={{ draft: "", working: allowed }}
            />
          </ul>
        </main>
      </div>
    </BotloftProvider>
  );
}
