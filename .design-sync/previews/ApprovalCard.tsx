import { ApprovalCard, BotloftProvider, chat, makeBot, makeCrew, schedule, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

const crew = makeCrew({ name: "Research" });
const chief = makeBot({ crew, name: "Chief", chief: true });
const scout = makeBot({ crew, name: "Scout", state: "needs_approval" });
const writer = makeBot({ crew, name: "Writer" });

function Card({ approval, bot }: { approval: Parameters<typeof ApprovalCard>[0]["approval"]; bot: typeof scout }) {
  return (
    <BotloftProvider crews={[crew]} bots={[chief, scout, writer]}>
      <div className="bg-canvas p-4" style={{ maxWidth: 640 }}>
        <ApprovalCard approval={approval} bot={bot} />
      </div>
    </BotloftProvider>
  );
}

export function Website() {
  return <Card bot={scout} approval={chat.askSite(scout, "https://arxiv.org/abs/2609.01234").body} />;
}

export function Command() {
  return (
    <Card
      bot={scout}
      approval={
        chat.askCommand(scout, "python charts/plot.py --week 40", {
          explanation: "Draws this week's benchmark chart for the summary.",
        }).body
      }
    />
  );
}

export function SuggestedBot() {
  return (
    <Card
      bot={chief}
      approval={
        chat.suggestBot(chief, {
          name: "Editor",
          role: "Checks facts and tone before the summary goes out.",
          model: "sonnet",
          instructions: "Read Writer's draft every Friday morning. Check each number against its source.",
          reason: "Summaries go out without a second look today.",
        }).body
      }
    />
  );
}

export function Plan() {
  return (
    <Card
      bot={chief}
      approval={
        chat.askPlan(
          chief,
          "## Friday summary\n\n1. Scout gathers sources Monday to Thursday\n2. Writer drafts on Thursday night\n3. Editor checks it Friday morning\n4. I send it to you at 9:00",
        ).body
      }
    />
  );
}

export function Allowed() {
  return (
    <Card bot={scout} approval={chat.askSite(scout, "https://arxiv.org/abs/2609.01234", { status: "allowed" }).body} />
  );
}

export function Routine() {
  return (
    <Card
      bot={chief}
      approval={
        chat.askRoutine(chief, {
          name: "Friday summary",
          prompt: "Write this week's summary from sources.md and send it to me.",
          schedule: schedule.weekly([5], "09:00"),
          bot: "writer",
        }).body
      }
    />
  );
}
