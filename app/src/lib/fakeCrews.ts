// The fake daemon's `crews.*` methods: names, slugs, the work folder,
// pausing, archiving and deleting, with the notifications each change sends.

import type { FakeBotloft, Handlers } from "./fake";
import { botHandlers } from "./fakeBots";
import { purgeBot } from "./fakeDelete";
import { checkName, invalid, slugify } from "./fakeRules";
import type { Crew } from "./protocol.gen";

type CrewMethods = Extract<keyof Handlers, `crews.${string}`>;

/** The crew's `shared` folder, where it works unless the owner chose another. */
const sharedFolder = (slug: string) => `C:\\Users\\owner\\Botloft\\${slug}\\shared`;

function checkFolder(folder: string): string {
  if (!/^[A-Za-z]:\\./.test(folder)) {
    throw invalid("the work folder must be a full path, like C:\\Projects\\Site");
  }
  return folder;
}

export function crewHandlers(fake: FakeBotloft): Pick<Handlers, CrewMethods> {
  return {
    "crews.list": () => [...fake.crews.values()].filter((crew) => crew.archivedAt === null),
    "crews.create": ({ name, workFolder, lead }) => {
      const checked = checkName(name);
      const slug = slugify(checked, "crew");
      const crew: Crew = {
        id: fake.id("crw"),
        name: checked,
        slug,
        workFolder: workFolder === undefined ? sharedFolder(slug) : checkFolder(workFolder),
        workFolderChosen: workFolder !== undefined,
        leadBotId: null,
        paused: false,
        createdAt: fake.now,
        archivedAt: null,
      };
      fake.crews.set(crew.id, crew);
      fake.changedCrew(crew);
      if (lead) {
        const chief = botHandlers(fake)["bots.create"]({ crewId: crew.id, ...lead });
        crew.leadBotId = chief.id;
        fake.changedCrew(crew);
      }
      return crew;
    },
    "crews.setLead": ({ crewId, botId }) => {
      const crew = fake.crew(crewId);
      if (botId !== null && fake.bot(botId).crewId !== crewId) {
        throw invalid(`bot ${botId} is not in crew ${crewId}`);
      }
      crew.leadBotId = botId;
      return fake.changedCrew(crew);
    },
    "crews.rename": ({ crewId, name }) => {
      const crew = fake.crew(crewId);
      crew.name = checkName(name);
      return fake.changedCrew(crew);
    },
    "crews.setPaused": ({ crewId, paused }) => {
      const crew = fake.crew(crewId);
      crew.paused = paused;
      return fake.changedCrew(crew);
    },
    "crews.setWorkFolder": ({ crewId, workFolder }) => {
      const crew = fake.crew(crewId);
      crew.workFolder = workFolder === null ? sharedFolder(crew.slug) : checkFolder(workFolder);
      crew.workFolderChosen = workFolder !== null;
      return fake.changedCrew(crew);
    },
    "crews.archive": ({ crewId }) => {
      const crew = fake.crew(crewId, false);
      if (crew.archivedAt === null) {
        crew.archivedAt = fake.now;
        for (const bot of fake.activeBots(crewId)) {
          bot.archivedAt = fake.now;
          bot.state = "archived";
          fake.changedBot(bot);
        }
        fake.changedCrew(crew);
      }
      return crew;
    },
    "crews.delete": ({ crewId, recycleFolder }) => {
      const crew = fake.crew(crewId, false);
      for (const bot of [...fake.bots.values()]) {
        if (bot.crewId === crewId) {
          purgeBot(fake, bot);
        }
      }
      fake.crews.delete(crewId);
      fake.emit({ name: "crew.deleted", params: { crewId } });
      if (recycleFolder) {
        fake.recycle(`C:\\Users\\owner\\Botloft\\${crew.slug}`);
      }
      return { crewId };
    },
  };
}
