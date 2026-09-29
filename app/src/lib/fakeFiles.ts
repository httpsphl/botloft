// The fake daemon's files: what a bot made, for the files panel.

import { encodeBytes } from "./base64";
import type { FakeBotloft, Handlers } from "./fake";
import { notFound } from "./fakeRules";
import type { BotFile, BotId, FileData } from "./protocol.gen";

const TYPES: Record<string, string> = {
  pdf: "application/pdf",
  png: "image/png",
  md: "text/markdown",
  txt: "text/plain",
  csv: "text/csv",
  docx: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
};

export class FakeFiles {
  private readonly files = new Map<BotId, BotFile[]>();
  private readonly contents = new Map<string, FileData>();

  constructor(private readonly fake: FakeBotloft) {}

  /**
   * A file the bot made. `text` is what its preview shows; `folder` is
   * relative to the work folder.
   */
  add(
    botId: BotId,
    name: string,
    options: { text?: string; folder?: string; writtenByBot?: boolean; at?: number } = {},
  ): BotFile {
    this.fake.bot(botId);
    const folder = options.folder ?? "";
    const text = options.text ?? "";
    const file: BotFile = {
      path: `C:\\Work\\${folder ? `${folder}\\` : ""}${name}`,
      name,
      folder,
      mediaType: TYPES[name.split(".").at(-1) ?? ""] ?? "application/octet-stream",
      size: text.length,
      modifiedAt: options.at ?? this.fake.now,
      writtenByBot: options.writtenByBot ?? false,
    };
    const list = (this.files.get(botId) ?? []).filter((other) => other.path !== file.path);
    this.files.set(botId, [file, ...list]);
    this.contents.set(file.path, {
      mediaType: file.mediaType,
      data: encodeBytes(new TextEncoder().encode(text)),
    });
    return file;
  }

  handlers(): Pick<Handlers, "files.list" | "files.read"> {
    return {
      "files.list": ({ botId }) => {
        this.fake.bot(botId, false);
        return [...(this.files.get(botId) ?? [])].sort((a, b) => b.modifiedAt - a.modifiedAt);
      },
      "files.read": ({ botId, path }) => {
        const known = (this.files.get(botId) ?? []).some((file) => file.path === path);
        const data = this.contents.get(path);
        if (!known || !data) {
          throw notFound(`file ${path}`);
        }
        return data;
      },
    };
  }
}
