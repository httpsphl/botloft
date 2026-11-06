// The fake daemon's side of the owner's skills (spec 38): the skills the
// owner has and which agents may use each one.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, notFound } from "./fakeRules";
import type { BotId, Skill } from "./protocol.gen";

export class FakeSkills {
  readonly skills: Skill[] = [];

  constructor(private readonly fake: FakeBotloft) {}

  /** A skill in the owner's folder, as the daemon would find it. */
  add(skill: Partial<Skill> & { name: string }): Skill {
    const made: Skill = {
      description: "",
      source: "personal",
      allowedTools: [],
      runsCommands: false,
      hasFiles: false,
      botIds: [],
      ...skill,
    };
    this.skills.push(made);
    this.skills.sort((a, b) => a.name.localeCompare(b.name));
    return made;
  }

  handlers(): Pick<Handlers, "skills.list" | "skills.share"> {
    return {
      "skills.list": () => ({
        skills: this.skills.map((skill) => ({
          ...skill,
          botIds: skill.botIds.filter((botId) => this.fake.bots.get(botId)?.archivedAt === null),
        })),
      }),
      "skills.share": ({ name, botIds }) => {
        const skill = this.skills.find((have) => have.name === name);
        if (!skill) {
          throw notFound(`skill ${name}`);
        }
        const wanted: BotId[] = [];
        for (const botId of botIds) {
          if (this.fake.bot(botId).agent !== "claude") {
            throw conflict("only agents on Claude Code can use skills");
          }
          if (!wanted.includes(botId)) {
            wanted.push(botId);
          }
        }
        skill.botIds = wanted;
        return { ...skill };
      },
    };
  }
}
