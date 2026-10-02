import {
  BotloftProvider,
  BotRun,
  BrowserPanel,
  chat,
  makeBot,
  makeCrew,
  setLocaleChoice,
  Sidebar,
} from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

const crew = makeCrew({ name: "Research" });
const chief = makeBot({ crew, name: "Chief", chief: true, state: "idle" });
const scout = makeBot({
  crew,
  name: "Scout",
  state: "busy",
  activity: { kind: "tool", text: "arxiv.org/abs/2609.01234", tool: "mcp__botloft__browser_open", minutesAgo: 0 },
});
const writer = makeBot({ crew, name: "Writer", state: "idle" });

// The app's window with Scout's browser open beside its chat: the sidebar,
// the chat column, then the panel, all direct children of one flex row.
export function BesideTheChat() {
  return (
    <BotloftProvider
      crews={[crew]}
      bots={[chief, scout, writer]}
      selectedBotId={scout.id}
      browsers={[
        {
          bot: scout,
          url: "https://arxiv.org/abs/2609.01234",
          title: "Small language models on phones: a benchmark",
          tabs: [{ url: "https://news.example/on-device-ai", title: "On-device AI this week" }],
          action: { kind: "scroll", x: 520, y: 360, label: "Abstract" },
        },
      ]}
    >
      <div className="flex border border-line bg-canvas text-ink" style={{ height: 600, width: 1400 }}>
        <Sidebar />
        <ul className="flex min-w-0 flex-1 flex-col gap-5 p-5">
          <BotRun
            bot={scout}
            items={[
              chat.reply(scout, "One paper looks key. I need to open it.", 1),
              chat.askSite(scout, "https://arxiv.org/abs/2609.01234", { status: "allowed", minutesAgo: 1 }),
              chat.tool(scout, { name: "mcp__botloft__browser_open", summary: "https://arxiv.org/abs/2609.01234" }),
            ]}
            live={{ draft: "", working: true }}
          />
        </ul>
        <BrowserPanel bot={scout} onClose={() => {}} />
      </div>
    </BotloftProvider>
  );
}
