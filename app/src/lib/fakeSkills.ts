// The fake daemon's side of the owner's skills (spec 38): the skills the
// owner has and which agents may use each one, chosen one by one or for
// every agent.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, notFound } from "./fakeRules";
import type { BotId, Skill, SkillList } from "./protocol.gen";

export class FakeSkills {
  /** Each with the agents chosen one by one in `botIds`. */
  readonly skills: Skill[] = [];
  /** Every skill for every agent on Claude Code. */
  everyone = false;

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
      everyone: false,
      tooBig: false,
      ...skill,
    };
    this.skills.push(made);
    this.skills.sort((a, b) => a.name.localeCompare(b.name));
    return made;
  }

  /** The skill as the daemon answers it: the agents that use it. */
  private shown(skill: Skill): Skill {
    const claude = [...this.fake.bots.values()]
      .filter((bot) => bot.archivedAt === null && bot.agent === "claude")
      .map((bot) => bot.id);
    const botIds = skill.tooBig
      ? []
      : this.everyone || skill.everyone
        ? claude
        : skill.botIds.filter((id) => claude.includes(id));
    return { ...skill, botIds };
  }

  private list(): SkillList {
    return { skills: this.skills.map((skill) => this.shown(skill)), everyone: this.everyone };
  }

  private find(name: string): Skill {
    const skill = this.skills.find((have) => have.name === name);
    if (!skill) {
      throw notFound(`skill ${name}`);
    }
    return skill;
  }

  handlers(): Pick<Handlers, "skills.list" | "skills.share" | "skills.everyone"> {
    return {
      "skills.list": () => this.list(),
      "skills.share": ({ name, botIds }) => {
        const skill = this.find(name);
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
        return this.shown(skill);
      },
      "skills.everyone": ({ name, on }) => {
        if (name === undefined) {
          this.everyone = on;
        } else {
          this.find(name).everyone = on;
        }
        return this.list();
      },
    };
  }
}
