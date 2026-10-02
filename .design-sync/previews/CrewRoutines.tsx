import {
  BotloftProvider,
  CrewRoutines,
  makeBot,
  makeCrew,
  makeRoutine,
  schedule,
  setLocaleChoice,
} from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

const crew = makeCrew({ name: "Research" });
const chief = makeBot({ crew, name: "Chief", chief: true });
const scout = makeBot({ crew, name: "Scout" });
const writer = makeBot({ crew, name: "Writer" });

const routines = [
  makeRoutine({
    bot: writer,
    name: "Friday summary",
    prompt: "Write this week's summary from sources.md and send it to me.",
    schedule: schedule.weekly([5], "09:00"),
    lastRun: { status: "done", minutesAgo: 60 * 24 * 5 },
  }),
  makeRoutine({
    bot: scout,
    name: "Source check",
    prompt: "Look for new papers and posts on on-device AI; add the good ones to sources.md.",
    schedule: schedule.every(120),
    lastRun: { status: "done", minutesAgo: 73 },
  }),
  makeRoutine({
    bot: chief,
    name: "Morning plan",
    prompt: "Check what each bot is on and tell me if anything is stuck.",
    schedule: schedule.weekdays("08:30"),
    lastRun: { status: "skipped", reason: "missed", skippedCount: 1, minutesAgo: 60 * 9 },
  }),
];

export function ResearchRoutines() {
  return (
    <BotloftProvider crews={[crew]} bots={[chief, scout, writer]} routines={routines}>
      <div className="flex flex-col bg-canvas" style={{ height: 360 }}>
        <CrewRoutines crew={crew} />
      </div>
    </BotloftProvider>
  );
}

export function Running() {
  const [friday, ...rest] = routines;
  const now = friday && { ...friday, lastRun: friday.lastRun && { ...friday.lastRun, status: "queued" as const, finishedAt: null } };
  return (
    <BotloftProvider crews={[crew]} bots={[chief, scout, writer]} routines={now ? [now, ...rest] : rest}>
      <div className="flex flex-col bg-canvas" style={{ height: 360 }}>
        <CrewRoutines crew={crew} />
      </div>
    </BotloftProvider>
  );
}

export function DarkTheme() {
  return (
    <BotloftProvider crews={[crew]} bots={[chief, scout, writer]} routines={routines}>
      <div data-theme="dark" className="flex flex-col bg-canvas text-ink" style={{ height: 360 }}>
        <CrewRoutines crew={crew} />
      </div>
    </BotloftProvider>
  );
}
