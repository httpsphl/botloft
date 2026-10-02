import { BotloftProvider, CrewBots, makeBot, makeCrew, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

const crew = makeCrew({ name: "Research" });
const bots = [
  makeBot({ crew, name: "Chief", chief: true, state: "idle", role: "Plans the work and keeps the crew on track." }),
  makeBot({ crew, name: "Scout", state: "busy", role: "Finds sources on on-device AI and passes the best ones on." }),
  makeBot({ crew, name: "Writer", state: "needs_approval", role: "Turns Scout's sources into the Friday summary." }),
  makeBot({ crew, name: "Editor", state: "offline", paused: true, role: "Checks facts and tone before it goes out." }),
];

export function ResearchCrew() {
  return (
    <BotloftProvider crews={[crew]} bots={bots}>
      <div className="bg-canvas p-4">
        <CrewBots crew={crew} bots={bots} onNewBot={() => {}} />
      </div>
    </BotloftProvider>
  );
}

export function DarkTheme() {
  return (
    <BotloftProvider crews={[crew]} bots={bots}>
      <div data-theme="dark" className="bg-canvas p-4 text-ink">
        <CrewBots crew={crew} bots={bots} onNewBot={() => {}} />
      </div>
    </BotloftProvider>
  );
}

export function Empty() {
  const fresh = makeCrew({ name: "Launch" });
  return (
    <BotloftProvider crews={[fresh]}>
      <div className="bg-canvas p-4">
        <CrewBots crew={fresh} bots={[]} onNewBot={() => {}} />
      </div>
    </BotloftProvider>
  );
}
