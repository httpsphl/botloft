import { BotloftProvider, InboundRow, makeBot, makeCrew, makeMessage, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

const crew = makeCrew({ name: "Research" });
const chief = makeBot({ crew, name: "Chief", chief: true });
const scout = makeBot({ crew, name: "Scout" });
const writer = makeBot({ crew, name: "Writer" });

export function FromAnotherBot() {
  return (
    <BotloftProvider crews={[crew]} bots={[chief, scout, writer]}>
      <ul className="flex flex-col gap-4 bg-canvas p-4" style={{ maxWidth: 720 }}>
        <InboundRow
          bot={writer}
          message={makeMessage({
            from: scout,
            to: writer,
            kind: "task",
            body: "Three sources for Friday's summary are in sources.md. The NPU benchmarks are the strongest.",
            minutesAgo: 2,
          })}
        />
      </ul>
    </BotloftProvider>
  );
}

export function FromYou() {
  return (
    <BotloftProvider crews={[crew]} bots={[chief, scout, writer]}>
      <ul className="flex flex-col gap-4 bg-canvas p-4" style={{ maxWidth: 720 }}>
        <InboundRow
          bot={chief}
          message={makeMessage({
            from: "owner",
            to: chief,
            body: "Follow what's new in on-device AI and send me a summary on Fridays.",
            minutesAgo: 5,
          })}
        />
      </ul>
    </BotloftProvider>
  );
}

export function FromBotloft() {
  return (
    <BotloftProvider crews={[crew]} bots={[chief, scout, writer]}>
      <ul className="flex flex-col gap-4 bg-canvas p-4" style={{ maxWidth: 720 }}>
        <InboundRow
          bot={scout}
          message={makeMessage({
            from: "system",
            to: scout,
            kind: "system",
            body: "Your usage limit resets at 4:10 PM. Scout continues on its own then.",
          })}
        />
      </ul>
    </BotloftProvider>
  );
}
