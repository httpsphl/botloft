// The fake daemon's history of each bot's memory and instructions (spec
// 8.10): versions newest first, and going back to one as a new version, like
// the daemon does.

import type { FakeBotloft, Handlers } from "./fake";
import { notFound } from "./fakeRules";
import type { BotId, TextVersion, VersionAuthor, VersionKind } from "./protocol.gen";

export class FakeVersions {
  /** Every version, oldest first. */
  private readonly all: TextVersion[] = [];

  constructor(private readonly fake: FakeBotloft) {}

  /** A new version, told to the app as the daemon would. */
  add(
    botId: BotId,
    kind: VersionKind,
    author: VersionAuthor,
    text: string,
    createdAt = this.fake.now,
    restoredFrom: string | null = null,
  ): TextVersion {
    const version: TextVersion = {
      id: this.fake.id("ver"),
      botId,
      kind,
      author,
      text,
      restoredFrom,
      createdAt,
    };
    this.all.push(version);
    this.fake.emit({ name: "version.created", params: version });
    return version;
  }

  handlers(): Pick<Handlers, "versions.list" | "versions.restore"> {
    return {
      "versions.list": ({ botId, kind }) => {
        this.fake.bot(botId);
        return this.all.filter((v) => v.botId === botId && v.kind === kind).reverse();
      },
      "versions.restore": ({ versionId }) => {
        const version = this.all.find((v) => v.id === versionId);
        if (!version) {
          throw notFound(`version ${versionId}`);
        }
        if (version.kind === "instructions") {
          this.fake.bot(version.botId).instructions = version.text;
        }
        return this.add(
          version.botId,
          version.kind,
          "owner",
          version.text,
          this.fake.now,
          version.id,
        );
      },
    };
  }
}
