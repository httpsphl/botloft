// The fake daemon's files: what a bot made, for the files panel, and the
// links its sounds and videos play from (spec 8.4).

import { encodeBytes } from "./base64";
import type { FakeBotloft, Handlers } from "./fake";
import { invalid, notFound } from "./fakeRules";
import type { BotFile, BotId, FileData } from "./protocol.gen";

const TYPES: Record<string, string> = {
  pdf: "application/pdf",
  png: "image/png",
  jpg: "image/jpeg",
  webp: "image/webp",
  md: "text/markdown",
  txt: "text/plain",
  csv: "text/csv",
  docx: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
  mp3: "audio/mpeg",
  wav: "audio/wav",
  m4a: "audio/mp4",
  mp4: "video/mp4",
  webm: "video/webm",
};

export class FakeFiles {
  private readonly files = new Map<BotId, BotFile[]>();
  private readonly contents = new Map<string, FileData>();
  /** Readable but not listed, like a file older than the bot (spec 8.4). */
  private readonly unlisted = new Set<string>();
  /** Where a sound or video plays from, when the preview has a real one. */
  private readonly media = new Map<string, string>();

  constructor(private readonly fake: FakeBotloft) {}

  /**
   * A file the bot made. `text` is what its preview shows, or `data` its
   * bytes in base64; `folder` is relative to the work folder. `listed:
   * false` leaves it out of the list, as the daemon does with a file older
   * than the bot.
   */
  add(
    botId: BotId,
    name: string,
    options: {
      text?: string;
      data?: string;
      size?: number;
      folder?: string;
      writtenByBot?: boolean;
      at?: number;
      listed?: boolean;
    } = {},
  ): BotFile {
    this.fake.bot(botId);
    const folder = options.folder ?? "";
    const text = options.text ?? "";
    const data = options.data ?? encodeBytes(new TextEncoder().encode(text));
    const file: BotFile = {
      path: `C:\\Work\\${folder ? `${folder}\\` : ""}${name}`,
      name,
      folder,
      mediaType: TYPES[name.split(".").at(-1)?.toLowerCase() ?? ""] ?? "application/octet-stream",
      size: options.size ?? (options.data ? Math.floor((data.length * 3) / 4) : text.length),
      modifiedAt: options.at ?? this.fake.now,
      writtenByBot: options.writtenByBot ?? false,
    };
    const list = (this.files.get(botId) ?? []).filter((other) => other.path !== file.path);
    this.files.set(botId, [file, ...list]);
    if (options.listed === false) {
      this.unlisted.add(file.path);
    }
    this.contents.set(file.path, { mediaType: file.mediaType, data });
    return file;
  }

  /** The address a sound or video of the preview really plays from. */
  play(path: string, url: string): void {
    this.media.set(path, url);
  }

  handlers(): Pick<Handlers, "files.list" | "files.read" | "files.media"> {
    const known = (botId: BotId, path: string) => {
      const file = (this.files.get(botId) ?? []).find((one) => one.path === path);
      const data = this.contents.get(path);
      if (!file || !data) {
        throw notFound(`file ${path}`);
      }
      return { file, data };
    };
    return {
      "files.list": ({ botId }) => {
        this.fake.bot(botId, false);
        return (this.files.get(botId) ?? [])
          .filter((file) => !this.unlisted.has(file.path))
          .sort((a, b) => b.modifiedAt - a.modifiedAt);
      },
      "files.read": ({ botId, path }) => known(botId, path).data,
      "files.media": ({ botId, path }) => {
        const { file } = known(botId, path);
        if (!/^(audio|video)\//.test(file.mediaType)) {
          throw invalid("only sound and video files play from a link");
        }
        return { url: this.media.get(path) ?? `http://127.0.0.1:45710/media/${file.name}` };
      },
    };
  }
}
